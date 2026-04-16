---

## name: apollo-config
version: 0.1.0
description: >-
  Query, read, and modify Apollo configuration center values using the apl CLI.
  Use when code references @Value, @ApolloJsonValue, Apollo dynamic config, or
  when you need actual runtime config values for code analysis. Also use when the
  user asks about Apollo config, wants to check/change config values, or mentions
  feature flags and dynamic thresholds.

# Apollo Configuration Lookup

Use the `apl` CLI to read and write Apollo configs during coding sessions.

## Prerequisites Check

Before any operation, verify setup:

```bash
which apl && cat .apollo-cli.toml 2>/dev/null || echo "NOT_CONFIGURED"
```

### If `apl` is not installed

Tell the user to install it:

```bash
cd apl-cli && cargo install --path .
```

### If `.apollo-cli.toml` is missing

Ask the user for these 4 values, then run init:

1. **portal_url** — Apollo Portal address (e.g. `http://apollo-portal.internal.com`)
2. **token** — Open API token (created in Apollo Portal → Open Platform)
3. **app_id** — application ID (e.g. `AppBitsfullWebService`)
4. **operator** — domain account / SSO username

Then execute:

```bash
apl init --portal-url "<url>" --token "<token>" --app-id "<appId>" --operator "<name>"
```

If the user provides the values in chat, construct and run the command yourself.

## Command Reference

### List namespaces

```bash
apl ns --format json
```

### Get all items in a namespace

```bash
apl get <namespace> --format json
```

### Get specific keys (most common — avoids context pollution)

```bash
apl get <namespace> --keys key1,key2,key3 --format json
```

### Get a single key

```bash
apl get <namespace> <key> --format json
```

### Set / update a value (non-PRO only)

```bash
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

## Important Rules

1. **Always use `--format json`** for machine-readable output
2. **Only fetch specific keys you need** — use `--keys k1,k2` to avoid flooding context
3. **PRO is read-only** — the CLI blocks all writes to PRO. Do not attempt `set` / `delete` / `publish` with `--env PRO`
4. **Confirm writes with user first** — before running `set` or `delete`, tell the user what you plan to change and get approval in chat. Then pass `--yes` to skip the interactive prompt
5. **Publish after set** — `set` only stages the change. Remind the user to `publish` if they want it to take effect immediately

## Typical Workflow

**Reading config for code analysis:**

```bash
apl get application --keys trade.order.max.retry,ws.reconnect.interval --format json
```

**Modifying a config value (after user approval):**

```bash
apl get application timeout --format json          # show current
apl set application timeout "5000" --comment "increase timeout per user request" --yes
apl publish application --title "update timeout" --yes
```

