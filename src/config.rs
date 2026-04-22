use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const CONFIG_FILE: &str = ".apollo-cli.toml";

/// Optional per-profile overrides. Unset fields inherit from the file root.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileOverlay {
    pub portal_url: Option<String>,
    pub token: Option<String>,
    pub default_env: Option<String>,
    pub default_app_id: Option<String>,
    pub default_cluster: Option<String>,
    pub default_operator: Option<String>,
    pub rate_limit_qps: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AplConfig {
    pub portal_url: Option<String>,
    pub token: Option<String>,
    pub default_env: Option<String>,
    pub default_app_id: Option<String>,
    pub default_cluster: Option<String>,
    pub default_operator: Option<String>,
    pub rate_limit_qps: Option<u32>,
    /// Named profiles, e.g. `[profiles.shared]` in TOML.
    #[serde(default)]
    pub profiles: std::collections::HashMap<String, ProfileOverlay>,
}

/// Effective file layer after optional profile merge (before CLI / env override).
#[derive(Debug, Clone)]
struct MergedFile {
    portal_url: Option<String>,
    token: Option<String>,
    default_env: Option<String>,
    default_app_id: Option<String>,
    default_cluster: Option<String>,
    default_operator: Option<String>,
    rate_limit_qps: Option<u32>,
}

impl AplConfig {
    pub fn path() -> PathBuf {
        PathBuf::from(CONFIG_FILE)
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(Self::path(), content)?;
        Ok(())
    }

    pub fn exists() -> bool {
        Self::path().exists()
    }

    /// Sorted profile names for `apl show --list-profiles` and error hints.
    pub fn profile_names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.profiles.keys().cloned().collect();
        v.sort();
        v
    }

    fn merge_profile(&self, name: &str) -> Result<MergedFile> {
        let p = self.profiles.get(name).with_context(|| {
            let avail = if self.profiles.is_empty() {
                "(no profiles in file; add [profiles.name] sections)".to_string()
            } else {
                self.profile_names().join(", ")
            };
            format!("Unknown profile: \"{name}\". Available: {avail}")
        })?;

        Ok(MergedFile {
            portal_url: p.portal_url.clone().or(self.portal_url.clone()),
            token: p.token.clone().or(self.token.clone()),
            default_env: p.default_env.clone().or(self.default_env.clone()),
            default_app_id: p.default_app_id.clone().or(self.default_app_id.clone()),
            default_cluster: p.default_cluster.clone().or(self.default_cluster.clone()),
            default_operator: p.default_operator.clone().or(self.default_operator.clone()),
            rate_limit_qps: p.rate_limit_qps.or(self.rate_limit_qps),
        })
    }

    fn merged_root(&self) -> MergedFile {
        MergedFile {
            portal_url: self.portal_url.clone(),
            token: self.token.clone(),
            default_env: self.default_env.clone(),
            default_app_id: self.default_app_id.clone(),
            default_cluster: self.default_cluster.clone(),
            default_operator: self.default_operator.clone(),
            rate_limit_qps: self.rate_limit_qps,
        }
    }
}

/// Fully resolved configuration with all required fields populated.
/// Priority: CLI flag > environment variable (handled by clap) > (profile-merged) config file > default.
pub struct Resolved {
    pub portal_url: String,
    pub token: String,
    pub env: String,
    pub app_id: String,
    pub cluster: String,
    pub operator: String,
    pub rate_limit_qps: u32,
    /// Set when `--profile` was used (or APOLLO_PROFILE), for display only.
    pub active_profile: Option<String>,
}

const DEFAULT_RATE_LIMIT_QPS: u32 = 10;

impl Resolved {
    #[allow(clippy::too_many_arguments)]
    pub fn from_cli(
        cli_profile: Option<&str>,
        cli_portal_url: Option<&str>,
        cli_token: Option<&str>,
        cli_env: Option<&str>,
        cli_app_id: Option<&str>,
        cli_cluster: Option<&str>,
        cli_operator: Option<&str>,
        cli_qps: Option<u32>,
    ) -> Result<Self> {
        let file = AplConfig::load()?;
        let merged = if let Some(pname) = cli_profile {
            file.merge_profile(pname)?
        } else {
            file.merged_root()
        };

        let active_profile = cli_profile.map(String::from);

        let portal_url = first_of(cli_portal_url, merged.portal_url.as_deref())
            .context("portal_url not set. Run: apl init --portal-url <url> --token <token> --app-id <id> or add a [profiles] section with portal_url+token+app_id")?;

        let token = first_of(cli_token, merged.token.as_deref())
            .context("token not set. Run: apl init ... or set token under [profiles]")?;

        let app_id = first_of(cli_app_id, merged.default_app_id.as_deref())
            .context("app_id not set. Run: apl init --app-id <id> or set default_app_id in file / profile")?;

        let env = first_of(cli_env, merged.default_env.as_deref()).unwrap_or("UAT".into());
        let cluster = first_of(cli_cluster, merged.default_cluster.as_deref()).unwrap_or("default".into());
        let operator = first_of(cli_operator, merged.default_operator.as_deref()).unwrap_or("apollo".into());
        let rate_limit_qps = cli_qps.or(merged.rate_limit_qps).unwrap_or(DEFAULT_RATE_LIMIT_QPS);

        Ok(Self {
            portal_url,
            token,
            env,
            app_id,
            cluster,
            operator,
            rate_limit_qps,
            active_profile,
        })
    }

    pub fn is_pro(&self) -> bool {
        let e = self.env.to_uppercase();
        e == "PRO" || e == "PROD" || e == "PRODUCTION"
    }
}

fn first_of(a: Option<&str>, b: Option<&str>) -> Option<String> {
    a.or(b).map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_profile_inherits_unset() {
        let mut cfg = AplConfig::default();
        cfg.portal_url = Some("http://p".into());
        cfg.token = Some("root-token".into());
        cfg.default_app_id = Some("AppA".into());
        cfg.profiles.insert(
            "b".into(),
            ProfileOverlay {
                default_app_id: Some("AppB".into()),
                token: Some("b-token".into()),
                ..Default::default()
            },
        );
        let m = cfg.merge_profile("b").unwrap();
        assert_eq!(m.portal_url.as_deref(), Some("http://p"));
        assert_eq!(m.token.as_deref(), Some("b-token"));
        assert_eq!(m.default_app_id.as_deref(), Some("AppB"));
    }
}
