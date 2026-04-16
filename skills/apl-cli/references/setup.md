# Apollo CLI Setup Guide

## Installing `apl`

Install with the official one-line script:

```bash
curl -fsSL https://raw.githubusercontent.com/AruNi-01/apl-cli/main/install.sh | sh
```

After installation, verify:

```bash
apl --version
```

## Initializing `.apollo-cli.toml`

Ask the user for these 4 values:

1. **portal_url** — Apollo Portal address (e.g. `http://apollo-portal.internal.com`)
2. **token** — Open API token (created in Apollo Portal → Open Platform)
3. **app_id** — application ID (e.g. `AppBitsfullWebService`)
4. **operator** — domain account / SSO username

Then execute:

```bash
apl init --portal-url "<url>" --token "<token>" --app-id "<appId>" --operator "<name>"
```

If the user provides the values in chat, construct and run the command yourself.
