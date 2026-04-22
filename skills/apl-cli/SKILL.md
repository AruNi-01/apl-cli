---
name: apl-cli
version: 0.3.0
description: Query, read, and modify Apollo configuration center values using the apl CLI. Use when code references @ApolloJsonValue, @ApolloConfig, @EnableApolloConfig, @ApolloConfigChangeListener, ConfigService, Config, or any Apollo-related annotation/class, or when the user mentions Apollo 配置, 配置中心, 开关, or wants to look up actual config values for code comprehension.
---

# Apollo Configuration Lookup

Use the `apl` CLI to read and write Apollo configs during coding sessions.

## Quick setup (default project / main Apollo app)

**Run this** before the first `apl` command in a session (main check):

```bash
command -v apl >/dev/null 2>&1 && test -f .apollo-cli.toml && echo "READY" || echo "NEED_SETUP"
```

- **READY** — continue with **Command reference** and **Typical workflow** below.
- **NEED_SETUP** — read `references/setup.md` in this skill directory, complete install or `apl init`, then re-run the check. **Do not** load `references/profiles.md` yet.

## Second Apollo app or different API token (profiles)

**Load `references/profiles.md` on demand** when (and only when) the user or task must access another project’s / another **app_id**’s namespace, or a **different Open API token** than the file root. Do not open that file for ordinary reads against the project’s default app.

**Run the profile check** from the project root (set the profile name to the one the team uses for the other app, e.g. `shared-infra`):

```bash
APOLLO_PROFILE_NAME=shared-infra
command -v apl >/dev/null 2>&1 \
  && test -f .apollo-cli.toml \
  && apl show --list-profiles 2>/dev/null | grep -qFx "$APOLLO_PROFILE_NAME" \
  && echo "READY" || echo "NEED_PROFILE"
```

- **READY** — use `--profile "$APOLLO_PROFILE_NAME"` (or `APOLLO_PROFILE`) on all `apl` invocations for that app; see `references/profiles.md` for command form.
- **NEED_PROFILE** — follow **`references/profiles.md`**: have the user supply `app_id` and token (and optional overrides), add `[profiles.…]` to `.apollo-cli.toml` or use one-off `--app-id` / `--token`, re-run the profile check until **READY**, then run the intended `apl` commands.

## Command reference

### List namespaces

```bash
apl ns --format json
```

With a profile:

```bash
apl ns --profile shared-infra --format json
```

### Get all items in a namespace

```bash
apl get <namespace> --format json
```

### Get specific keys (preferred — avoids context pollution)

```bash
apl get <namespace> --keys key1,key2,key3 --format json
```

### Get a single key

```bash
apl get <namespace> <key> --format json
```

### Set / update a value (non-PRO only)

```bash
# Updating an existing key — do not pass `--comment` (CLI ignores it and keeps the portal remark)
apl set <namespace> <key> "<value>" --yes

# Creating a new key — optional `--comment` documents the item in Apollo
apl set <namespace> <key> "<value>" --comment "reason" --yes
```

### Delete a key (non-PRO only)

```bash
apl delete <namespace> <key> --yes
```

### Publish changes (non-PRO only)

```bash
apl publish <namespace> --title "description" --yes
```

### Switch environment

```bash
apl get <namespace> --env FAT --format json
```

### Show help

```bash
apl --help            # main help
apl <command> --help  # subcommand help
```

## Important rules

1. **Always use `--format json`** for machine-readable output.
2. **Only fetch specific keys you need** — use `--keys k1,k2` to avoid flooding context.
3. **PRO is read-only** — the CLI blocks all writes to PRO. Do not attempt `set` / `delete` / `publish` with `--env PRO`.
4. **Confirm writes with user first** — before running `set` or `delete`, tell the user what you plan to change and get approval. Then pass `--yes` to skip the interactive prompt.
5. **Do not overwrite item remarks (备注)** — `--comment` on `apl set` applies **only when the key is new**. For existing keys, the CLI **never** applies your `--comment` to the item; it keeps the remark already stored in Apollo. Agents must **not** pass `--comment` when updating an existing key (it is ignored and would mislead readers of the command).
6. **Publish after set** — `set` only stages the change. Remind the user to `publish` if they want it to take effect immediately.
7. **Rate limiting is built-in** — default 10 QPS, configurable via `rate_limit_qps` in `.apollo-cli.toml` or `--qps` flag. No need to add external throttling.
8. **Another app / token** — use `--profile <name>` when configured, or one-off `--app-id` and `--token`. Details and NEED_PROFILE flow: **`references/profiles.md`** (load when needed only).

## Typical workflow

**Reading config for code analysis:**

```bash
apl get application --keys trade.order.max.retry,ws.reconnect.interval --format json
```

**Modifying an existing config value (after user approval):**

```bash
apl get application timeout --format json
apl set application timeout "5000" --yes
apl publish application --title "update timeout" --yes
```

**Adding a new key (optional `--comment` for the new item only):**

```bash
apl set application feature.new.flag "true" --comment "rollout flag" --yes
apl publish application --title "add feature flag" --yes
```
