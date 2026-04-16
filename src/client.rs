use std::num::NonZeroU32;

use anyhow::{bail, Result};
use governor::{Quota, RateLimiter, clock::{Clock, DefaultClock}};
use reqwest::blocking::Client;
use reqwest::StatusCode;

use crate::config::Resolved;
use crate::models::*;

type DirectLimiter = RateLimiter<
    governor::state::NotKeyed,
    governor::state::InMemoryState,
    DefaultClock,
>;

pub struct ApolloClient {
    http: Client,
    base: String,
    token: String,
    pub env: String,
    pub app_id: String,
    pub cluster: String,
    limiter: DirectLimiter,
}

impl ApolloClient {
    pub fn new(cfg: &Resolved) -> Self {
        let portal = cfg.portal_url.trim_end_matches('/');
        let qps = NonZeroU32::new(cfg.rate_limit_qps).unwrap_or(NonZeroU32::new(10).unwrap());
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client"),
            base: format!("{}/openapi/v1", portal),
            token: cfg.token.clone(),
            env: cfg.env.clone(),
            app_id: cfg.app_id.clone(),
            cluster: cfg.cluster.clone(),
            limiter: RateLimiter::direct(Quota::per_second(qps)),
        }
    }

    // ── Read APIs ──────────────────────────────────────────────

    pub fn env_clusters(&self) -> Result<Vec<EnvCluster>> {
        let url = format!("{}/apps/{}/envclusters", self.base, self.app_id);
        self.get_json(&url)
    }

    pub fn list_namespaces(&self) -> Result<Vec<NamespaceInfo>> {
        let url = format!(
            "{}/envs/{}/apps/{}/clusters/{}/namespaces",
            self.base, self.env, self.app_id, self.cluster
        );
        self.get_json(&url)
    }

    pub fn get_namespace(&self, ns: &str) -> Result<NamespaceInfo> {
        let url = format!(
            "{}/envs/{}/apps/{}/clusters/{}/namespaces/{}",
            self.base, self.env, self.app_id, self.cluster, ns
        );
        self.get_json(&url)
    }

    pub fn get_item(&self, ns: &str, key: &str) -> Result<ConfigItem> {
        let url = format!(
            "{}/envs/{}/apps/{}/clusters/{}/namespaces/{}/items/{}",
            self.base, self.env, self.app_id, self.cluster, ns, key
        );
        self.get_json(&url)
    }

    /// Returns None on 404, errors on other failures.
    pub fn try_get_item(&self, ns: &str, key: &str) -> Result<Option<ConfigItem>> {
        let url = format!(
            "{}/envs/{}/apps/{}/clusters/{}/namespaces/{}/items/{}",
            self.base, self.env, self.app_id, self.cluster, ns, key
        );
        let resp = self.request(reqwest::Method::GET, &url)?.send()?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !resp.status().is_success() {
            bail!("{}", Self::fmt_error(resp));
        }
        Ok(Some(resp.json()?))
    }

    // ── Write APIs ─────────────────────────────────────────────

    #[allow(dead_code)]
    pub fn create_item(&self, ns: &str, req: &CreateItemRequest) -> Result<ConfigItem> {
        let url = format!(
            "{}/envs/{}/apps/{}/clusters/{}/namespaces/{}/items",
            self.base, self.env, self.app_id, self.cluster, ns
        );
        let resp = self.request(reqwest::Method::POST, &url)?.json(req).send()?;
        if !resp.status().is_success() {
            bail!("{}", Self::fmt_error(resp));
        }
        Ok(resp.json()?)
    }

    pub fn update_item(
        &self,
        ns: &str,
        key: &str,
        req: &UpdateItemRequest,
        create_if_not_exists: bool,
    ) -> Result<()> {
        let mut url = format!(
            "{}/envs/{}/apps/{}/clusters/{}/namespaces/{}/items/{}",
            self.base, self.env, self.app_id, self.cluster, ns, key
        );
        if create_if_not_exists {
            url.push_str("?createIfNotExists=true");
        }
        let resp = self.request(reqwest::Method::PUT, &url)?.json(req).send()?;
        if !resp.status().is_success() {
            bail!("{}", Self::fmt_error(resp));
        }
        Ok(())
    }

    pub fn delete_item(&self, ns: &str, key: &str, operator: &str) -> Result<()> {
        let url = format!(
            "{}/envs/{}/apps/{}/clusters/{}/namespaces/{}/items/{}?operator={}",
            self.base, self.env, self.app_id, self.cluster, ns, key, operator
        );
        let resp = self.request(reqwest::Method::DELETE, &url)?.send()?;
        if !resp.status().is_success() {
            bail!("{}", Self::fmt_error(resp));
        }
        Ok(())
    }

    pub fn publish(&self, ns: &str, req: &PublishRequest) -> Result<ReleaseInfo> {
        let url = format!(
            "{}/envs/{}/apps/{}/clusters/{}/namespaces/{}/releases",
            self.base, self.env, self.app_id, self.cluster, ns
        );
        let resp = self.request(reqwest::Method::POST, &url)?.json(req).send()?;
        if !resp.status().is_success() {
            bail!("{}", Self::fmt_error(resp));
        }
        Ok(resp.json()?)
    }

    // ── Helpers ────────────────────────────────────────────────

    fn wait_for_permit(&self) {
        let clock = DefaultClock::default();
        loop {
            match self.limiter.check() {
                Ok(_) => return,
                Err(not_until) => {
                    std::thread::sleep(not_until.wait_time_from(clock.now()));
                }
            }
        }
    }

    fn request(
        &self,
        method: reqwest::Method,
        url: &str,
    ) -> Result<reqwest::blocking::RequestBuilder> {
        self.wait_for_permit();
        Ok(self
            .http
            .request(method, url)
            .header("Authorization", &self.token)
            .header("Content-Type", "application/json;charset=UTF-8"))
    }

    fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = self.request(reqwest::Method::GET, url)?.send()?;
        if !resp.status().is_success() {
            bail!("{}", Self::fmt_error(resp));
        }
        Ok(resp.json()?)
    }

    fn fmt_error(resp: reqwest::blocking::Response) -> String {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        match status {
            StatusCode::UNAUTHORIZED => {
                "401 Unauthorized: token invalid or expired".into()
            }
            StatusCode::FORBIDDEN => {
                format!("403 Forbidden: no permission for this namespace. {body}")
            }
            StatusCode::NOT_FOUND => {
                format!("404 Not Found: check appId / env / namespace / key. {body}")
            }
            _ => format!("HTTP {}: {body}", status.as_u16()),
        }
    }
}
