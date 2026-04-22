# Apollo: second app / different Open API token (profiles)

Use this **only** when the task must read or change configuration under **another** Apollo `app_id` than the one in the project root, or when that app requires a **different Open API token** in Portal.

## Before any `apl` command against the other app

1. **Choose a stable profile name** (ASCII: letters, numbers, `-`, `_`, `.`), e.g. `shared-infra` or `partner-api`. Use the same name in checks and in `.apollo-cli.toml`.
2. Run the **profile readiness check** from the project root (set `APOLLO_PROFILE_NAME` to that name).

```bash
APOLLO_PROFILE_NAME=shared-infra
command -v apl >/dev/null 2>&1 \
  && test -f .apollo-cli.toml \
  && apl show --list-profiles 2>/dev/null | grep -qFx "$APOLLO_PROFILE_NAME" \
  && echo "READY" || echo "NEED_PROFILE"
```

- **READY** — run normal commands with `--profile "$APOLLO_PROFILE_NAME"` (or `APOLLO_PROFILE` in the environment). See **Command form** below.
- **NEED_PROFILE** — the CLI or config file is missing, or the named `[profiles.…]` block does not exist. Do **not** continue with a guessed token.

## If output is NEED_PROFILE

### A. `apl` missing or not executable

Use `references/setup.md` (main skill directory) to install, then re-run the check.

### B. `.apollo-cli.toml` missing

Run `apl init` with the **root** (default) app first, or follow `references/setup.md`. Root config is still required; profiles are extra sections in the same file.

### C. File exists but profile is absent

Add a block under the project’s `.apollo-cli.toml` (ask the user for values you cannot infer):

- **`default_app_id`** — the other application’s id in Apollo.
- **`token`** — Open API token that Portal has authorized for **that** app (not necessarily the same as the root `token`).

Optional overrides in the same block: `portal_url`, `default_env`, `default_cluster`, `default_operator`, `rate_limit_qps` — only if the other app or environment differs from the file root.

Example:

```toml
# Existing root keys stay as the main project
portal_url     = "http://apollo-portal.internal.example"
token          = "token-for-service-a"
default_app_id = "ServiceA"
default_env    = "UAT"

[profiles.shared-infra]
default_app_id = "InfraApp"
token          = "open-api-token-authorized-for-InfraApp"
```

**Security:** do not commit real tokens. Prefer local edit or team secrets. After editing, re-run the profile check until it prints **READY**.

## Command form (after READY)

Always scope the other app with `--profile`:

```bash
apl get <namespace> --profile "$APOLLO_PROFILE_NAME" --keys k1,k2 --format json
```

One-off without editing the file (same precedence as the CLI help):

```bash
apl get <namespace> --app-id OtherAppId --token "<other-open-api-token>" --format json
```

## List configured profile names

```bash
apl show --list-profiles
```

## Inspect merged resolution (masked token)

```bash
apl show --profile "$APOLLO_PROFILE_NAME"
```
