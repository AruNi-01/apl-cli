use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "apl", version, about = "Apollo Configuration Center CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Apollo Portal URL
    #[arg(long, global = true, env = "APOLLO_PORTAL_URL")]
    pub portal_url: Option<String>,

    /// Authentication token
    #[arg(long, global = true, env = "APOLLO_TOKEN")]
    pub token: Option<String>,

    /// Environment (DEV/FAT/UAT/PRO)
    #[arg(long, global = true, env = "APOLLO_ENV")]
    pub env: Option<String>,

    /// Application ID
    #[arg(long, global = true, env = "APOLLO_APP_ID")]
    pub app_id: Option<String>,

    /// Cluster name
    #[arg(long, global = true, env = "APOLLO_CLUSTER")]
    pub cluster: Option<String>,

    /// Rate limit: max queries per second (default: 10)
    #[arg(long, global = true)]
    pub qps: Option<u32>,

    /// Output format
    #[arg(long, global = true, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Write .apollo-cli.toml configuration file in the current directory
    Init {
        /// Apollo Portal URL
        #[arg(long)]
        portal_url: String,
        /// Authentication token
        #[arg(long)]
        token: String,
        /// Default environment (DEV/FAT/UAT/PRO)
        #[arg(long, default_value = "UAT")]
        env: String,
        /// Default application ID
        #[arg(long)]
        app_id: String,
        /// Default cluster name
        #[arg(long, default_value = "default")]
        cluster: String,
        /// Default operator (domain account)
        #[arg(long, default_value = "apollo")]
        operator: String,
        /// Rate limit: max queries per second (default: 10)
        #[arg(long, default_value = "10")]
        qps: u32,
    },
    /// Show current resolved configuration (optionally filter by field name)
    Show {
        /// Config field to show (e.g. portal_url, token, env, app_id, cluster, operator, qps)
        field: Option<String>,
    },
    /// List environments and clusters for the app
    Envs,
    /// List all namespaces under the app
    Ns,
    /// Get configuration value(s) from a namespace
    Get {
        /// Namespace name
        namespace: String,
        /// Single key to retrieve (positional)
        key: Option<String>,
        /// Comma-separated keys to filter
        #[arg(long)]
        keys: Option<String>,
    },
    /// Create or update a configuration item (blocked for PRO)
    Set {
        /// Namespace name
        namespace: String,
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
        /// Comment for the item
        #[arg(long)]
        comment: Option<String>,
        /// Operator (domain account)
        #[arg(long)]
        operator: Option<String>,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
    /// Delete a configuration item (blocked for PRO)
    Delete {
        /// Namespace name
        namespace: String,
        /// Configuration key
        key: String,
        /// Operator (domain account)
        #[arg(long)]
        operator: Option<String>,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
    /// Upgrade apl to the latest version
    Upgrade,
    /// Publish namespace changes (blocked for PRO)
    Publish {
        /// Namespace name
        namespace: String,
        /// Release title
        #[arg(long)]
        title: Option<String>,
        /// Release comment
        #[arg(long)]
        comment: Option<String>,
        /// Operator (domain account)
        #[arg(long)]
        operator: Option<String>,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}
