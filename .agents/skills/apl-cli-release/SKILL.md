---
name: apl-cli-release
version: 0.1.1
description: >-
  Automate the release process for apl-cli. Use this skill whenever the user
  says "release", "发布", "打 tag", "publish a new version", "bump version",
  "发新版", or any phrase indicating they want to cut a new release. Also use
  when the user references this skill by name.
---

# apl-cli Release

Automate version bump, changelog generation, and GitHub Release for the apl-cli project.

## Pre-flight Checks

Before anything else, verify the repo is in a releasable state:

```bash
git status --porcelain
```

- If there are uncommitted changes, ask the user whether to commit them first or abort.
- If on a branch other than `main`, warn the user and confirm before proceeding.

## Step 1 — Determine the Last Tag

```bash
git describe --tags --abbrev=0
```

This gives the most recent tag (e.g. `v0.2.4`). All subsequent analysis is relative to this tag.

## Step 2 — Collect Commits Since Last Tag

```bash
git log <last-tag>..HEAD --oneline
```

If there are **zero** commits since the last tag, tell the user there is nothing to release and stop.

## Step 3 — Determine Version Bump

Version format: **MAJOR.MINOR.PATCH** (e.g. `2.5.13`)

Do NOT blindly follow the commit prefix. Read the actual changes and judge by their **user-facing impact**:

| What changed                                         | Bump  | Example                          |
| ---------------------------------------------------- | ----- | -------------------------------- |
| Breaking change (incompatible API, config format, CLI args removed) | MAJOR | change API response structure: `1.4.2` → `2.0.0` |
| New user-facing feature (new command, new capability) | MINOR | add export function: `1.4.2` → `1.5.0` |
| Bug fix, performance improvement, internal refactor, docs, CI, tooling, adding skill files, dependency updates | PATCH | fix login bug: `1.4.2` → `1.4.3` |

Key distinctions:
- `feat:` in the commit message does **not** automatically mean MINOR. A commit tagged `feat:` that only adds internal tooling, documentation, or skill files is still PATCH.
- MINOR is reserved for changes that give users **new functionality they can use** (e.g. a new CLI command, a new flag, a new config option).
- When in doubt, prefer PATCH.

Take the **highest** applicable bump across all commits. Calculate the new version from the last tag accordingly.

## Step 4 — Check Skill Changes (independent semver)

```bash
git diff <last-tag>..HEAD --name-only -- skills/
```

The **`version:` in `skills/apl-cli/SKILL.md` is the skill’s own semver**. It does **not** have to match the CLI / `Cargo.toml` version.

- If **nothing** under `skills/` changed since `<last-tag>`, do **not** bump the skill `version:` in this release.
- If **any** file under `skills/` changed, bump `skills/apl-cli/SKILL.md` frontmatter `version:`:
  1. Read the **current** skill `version:` from that file.
  2. Inspect the `skills/` diff and choose **MAJOR / MINOR / PATCH** from **skill-consumer impact** (instructions, examples, rules — not the Rust binary):
     | What changed in `skills/` | Bump |
     | ------------------------- | ---- |
     | Breaking for agents (removed commands, reversed required flow, renamed critical rules) | MAJOR |
     | New documented capability, new section, new command examples that change how agents work | MINOR |
     | Typos, clarifications, small rule tweaks, wording | PATCH |
  3. When in doubt, prefer **PATCH** for the skill.
  4. Compute the new skill version from the **existing skill `version:`**, not from the CLI tag.

**Never** set skill `version:` equal to the CLI version just to “keep them in sync.”

## Step 5 — Apply Version Bumps

1. **Cargo.toml** — update the `version = "..."` field to the **new CLI version** from Step 3 (this is what the `v*` tag represents).
2. **skills/apl-cli/SKILL.md** — only if Step 4 required it: update frontmatter `version:` to the **new skill version** from Step 4 (independent of `Cargo.toml`).

After editing, do a quick sanity check:

```bash
cargo check
```

## Step 6 — Generate Release Notes

Build a human-readable changelog from the commits collected in Step 2. Group by type:

```markdown
## What's Changed

### Features
- support `apl show <field>` to query a single config value (2f8c7f2)

### Bug Fixes
- handle GitHub API rate limit and auto-detect gh CLI token (8168e12)
- sync entire skill directory on upgrade, not just SKILL.md (27848ed)

### Other
- bump version to 0.2.4 (89cae08)
```

Rules:
- Use the **commit subject** (first line) as the description, with the short hash in parentheses.
- Omit the conventional commit prefix from the description (e.g. show "add foo" not "feat: add foo").
- Group mapping: `feat` → Features, `fix` → Bug Fixes, `docs` → Documentation, `perf` → Performance. Everything else (`chore`, `refactor`, `test`, `ci`, `build`, `style`) → Other.
- Skip empty groups.
- Append a **Full Changelog** link at the bottom: `**Full Changelog**: https://github.com/AruNi-01/apl-cli/compare/<old-tag>...v<new-version>`

## Step 7 — Commit, Tag, Push

Run these sequentially:

```bash
git add -A
git commit -m "chore(release): v<new-version>"
git push origin main
git tag -a v<new-version> -m "release v<new-version>"
git push origin v<new-version>
```

## Step 8 — Create GitHub Release

Use `gh release create` with the generated notes. Do **not** use `--generate-notes` — use our own changelog from Step 6.

```bash
gh release create v<new-version> \
  --title "v<new-version>" \
  --notes "$(cat <<'EOF'
<release notes from Step 6>
EOF
)"
```

Wait, then confirm the release URL is accessible:

```bash
gh release view v<new-version> --json url -q .url
```

## Step 9 — Report

Print a summary to the user:

```
Release complete!
  Version : v<old> -> v<new>   (CLI / tag)
  Skill   : <old-skill-ver> -> <new-skill-ver>   (only if skills/ bumped)
  Commits : <count>
  Tag     : v<new-version>
  Release : <github-release-url>
```

Omit the `Skill` line if `skills/` was not bumped this release.

## Important

- Always bump `Cargo.toml` version **before** tagging — the version is baked into the binary at compile time.
- **Skill semver** (`skills/apl-cli/SKILL.md` `version:`) is separate from the CLI; bump it only from Step 4 rules when `skills/` changes.
- The CI workflow (`.github/workflows/release.yml`) is triggered by `v*` tags and builds release binaries automatically. The `gh release create` here creates the Release object with proper notes; CI will attach the binaries to it.
- If CI hasn't finished attaching binaries yet when the release is created, that's fine — they'll appear once the workflow completes.
