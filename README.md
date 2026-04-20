# APL-CLI

**Apollo Configuration Center CLI**

[简体中文](./README.zh-CN.md) | English

Command-line interface for the [Apollo](https://www.apolloconfig.com/) configuration center. Use it to read and manage dynamic configuration from the terminal—especially in AI-assisted coding workflows where agents need real values instead of placeholders.

## Installation

### One-line install (recommended, no Rust)

```bash
curl -fsSL https://raw.githubusercontent.com/AruNi-01/apl-cli/main/install.sh | sh
```

Detects OS (macOS / Linux) and architecture (x86_64 / aarch64), then installs the prebuilt binary to `~/.local/bin`.

### Install via Skill

For agents that support Skills, install the bundled skill first:

```bash
npx skills add https://github.com/AruNi-01/apl-cli
```

Then prompt the agent, for example:

```text
Use the apl-cli skill to install the CLI and run setup initialization.
```

The agent checks for `apl` on the machine; if missing, it runs the install script and walks you through Apollo configuration.

### Build from source

**Prerequisites:** Rust toolchain with `cargo` (stable recommended).

```bash
cargo install --git https://github.com/AruNi-01/apl-cli.git
```

### Verify

```bash
apl --version
```

## Quick start

### 1. Create a token

In Apollo Portal, create an Open API token:

> Apollo Portal → Open Platform → Create third-party application → Authorize namespaces

### 2. Initialize configuration

From your project root:

```bash
apl init \
  --portal-url "http://apollo-portal.your-company.com" \
  --token "your-open-api-token" \
  --app-id "YourAppId" \
  --operator "your-domain-account"
```

This writes `.apollo-cli.toml` in the current directory—one config per project.

### 3. Use the CLI

```bash
# List namespaces
apl ns

# Read all keys in a namespace
apl get application

# Read selected keys (common for agents—reduces noise)
apl get application --keys timeout,batch.size,retry.count

# Read a single key
apl get application timeout
```

## Commands

| Command | Description |
| --- | --- |
| `apl init` | Create `.apollo-cli.toml` |
| `apl show` | Show current config (token masked) |
| `apl envs` | List environments and clusters |
| `apl ns` | List namespaces |
| `apl get <ns> [key]` | Read config; supports `--keys k1,k2` |
| `apl set <ns> <key> <value>` | Create or update a key |
| `apl delete <ns> <key>` | Delete a key |
| `apl publish <ns>` | Publish namespace changes |
| `apl upgrade` | Upgrade to the latest release |

## Reading configuration

```bash
# Entire namespace
apl get application

# Multiple keys
apl get application --keys timeout,max.retry

# Single key
apl get application timeout

# JSON output (recommended for agents)
apl get application --keys timeout,batch --format json
# Example: {"batch":"100","timeout":"3000"}

# Another environment
apl get application --env FAT --format json
```

## Changing configuration

```bash
# Update a value (confirmation prompt; `--comment` applies when creating a new key only—existing keys keep Portal notes)
apl set application timeout 5000 --yes

# New key with a comment (omit `--comment` when updating an existing key)
apl set application new.feature.flag true --comment "rollout flag" --yes

# Publish so changes take effect
apl publish application --title "update timeout"
```

**Production (PRO) guard:** `set`, `delete`, and `publish` are blocked in PRO. Use Apollo Portal for writes in production.

## Configuration file

Path: project root `.apollo-cli.toml`

```toml
portal_url       = "http://apollo-portal.your-company.com"
token            = "your-open-api-token"
default_env      = "UAT"
default_app_id   = "YourAppId"
default_cluster  = "default"
default_operator = "your-domain-account"
rate_limit_qps   = 10
```

**Precedence:** CLI flags > environment variables > config file > defaults

Environment variables:

| Variable | Maps to |
| --- | --- |
| `APOLLO_PORTAL_URL` | `portal_url` |
| `APOLLO_TOKEN` | `token` |
| `APOLLO_ENV` | `default_env` |
| `APOLLO_APP_ID` | `default_app_id` |
| `APOLLO_CLUSTER` | `default_cluster` |

## Global options

All commands accept:

```
--portal-url <URL>     Override Portal URL
--token <TOKEN>        Override token
--env <ENV>            Override environment (DEV/FAT/UAT/PRO)
--app-id <ID>          Override AppId
--cluster <NAME>       Override cluster (default: default)
--qps <N>              Override client QPS (default: 10)
--format <text|json>   Output format (default: text)
```

## Rate limiting

HTTP calls are client-side rate limited ([governor](https://crates.io/crates/governor), GCRA). Default **10 QPS**; adjust via:

```bash
# Config file (.apollo-cli.toml)
rate_limit_qps = 5

# CLI (higher precedence)
apl ns --qps 5

# During init
apl init --portal-url "..." --token "..." --app-id "..." --qps 5
```

When over the limit, the CLI waits until the next window automatically.

## Auto-update

About once every 24 hours, any command may check GitHub Releases and print a hint after output, for example:

```
New version available: 0.2.0 -> 0.3.0 (run apl upgrade to upgrade)
```

Upgrade:

```bash
apl upgrade
```

Downloads the latest binary for your platform and replaces the current executable.

## AI agent integration

With the Skill installed (`~/.agents/skills/apl-cli/SKILL.md`), agents typically use the CLI when:

- Code uses `@Value("${...}")` or `@ApolloJsonValue` and needs real values
- You ask about Apollo or dynamic configuration
- Analysis needs runtime settings (feature flags, thresholds, URLs, …)

Agents should prefer `--format json` and `--keys` to keep context small.

## Releasing

Push a `v*` tag; GitHub Actions builds four targets and creates a Release:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

| Platform | Build |
| --- | --- |
| Linux x86_64 | ubuntu-latest, native |
| Linux aarch64 | cross-compile |
| macOS x86_64 (Intel) | macos-15-intel, native |
| macOS aarch64 (Apple Silicon) | macos-latest, native |

## Repository layout

```
apl-cli/
├── .github/workflows/
│   └── release.yml     # CI: tag → build → Release
├── Cargo.toml
├── install.sh
├── LICENSE
├── README.md
├── README.zh-CN.md
├── skills/
│   └── apl-cli/
│       └── SKILL.md    # Agent Skill (source)
└── src/
    ├── main.rs
    ├── cli.rs
    ├── config.rs
    ├── client.rs
    ├── models.rs
    ├── output.rs
    ├── upgrade.rs
    └── commands.rs
```

## License

MIT. See [LICENSE](./LICENSE).
