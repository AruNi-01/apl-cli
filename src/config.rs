use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const CONFIG_FILE: &str = ".apollo-cli.toml";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AplConfig {
    pub portal_url: Option<String>,
    pub token: Option<String>,
    pub default_env: Option<String>,
    pub default_app_id: Option<String>,
    pub default_cluster: Option<String>,
    pub default_operator: Option<String>,
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
}

/// Fully resolved configuration with all required fields populated.
/// Priority: CLI flag > env var (handled by clap) > config file > default.
pub struct Resolved {
    pub portal_url: String,
    pub token: String,
    pub env: String,
    pub app_id: String,
    pub cluster: String,
    pub operator: String,
}

impl Resolved {
    pub fn from_cli(
        cli_portal_url: Option<&str>,
        cli_token: Option<&str>,
        cli_env: Option<&str>,
        cli_app_id: Option<&str>,
        cli_cluster: Option<&str>,
        cli_operator: Option<&str>,
    ) -> Result<Self> {
        let file = AplConfig::load()?;

        let portal_url = first_of(cli_portal_url, file.portal_url.as_deref())
            .context("portal_url not set. Run: apl init --portal-url <url> --token <token> --app-id <id>")?;

        let token = first_of(cli_token, file.token.as_deref())
            .context("token not set. Run: apl init --portal-url <url> --token <token> --app-id <id>")?;

        let app_id = first_of(cli_app_id, file.default_app_id.as_deref())
            .context("app_id not set. Run: apl init --portal-url <url> --token <token> --app-id <id>")?;

        let env = first_of(cli_env, file.default_env.as_deref()).unwrap_or("UAT".into());
        let cluster = first_of(cli_cluster, file.default_cluster.as_deref()).unwrap_or("default".into());
        let operator = first_of(cli_operator, file.default_operator.as_deref()).unwrap_or("apollo".into());

        Ok(Self { portal_url, token, env, app_id, cluster, operator })
    }

    pub fn is_pro(&self) -> bool {
        let e = self.env.to_uppercase();
        e == "PRO" || e == "PROD" || e == "PRODUCTION"
    }
}

fn first_of(a: Option<&str>, b: Option<&str>) -> Option<String> {
    a.or(b).map(String::from)
}
