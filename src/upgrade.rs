use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use colored::Colorize;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

const REPO: &str = "AruNi-01/apl-cli";
const CHECK_INTERVAL_SECS: u64 = 86400; // 24h
const GITHUB_TIMEOUT_SECS: u64 = 3;
const SKILLS_DIR: &str = "skills";
const RAW_BASE: &str = "https://raw.githubusercontent.com/AruNi-01/apl-cli/main";

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
}

#[derive(Serialize, Deserialize)]
struct VersionCache {
    last_check: u64,
    latest_version: String,
}

fn cache_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".apl-cli")
}

fn cache_path() -> PathBuf {
    cache_dir().join("version-cache.json")
}

fn now_epoch() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn github_token() -> Option<String> {
    if let Ok(t) = env::var("GITHUB_TOKEN") {
        if !t.is_empty() {
            return Some(t);
        }
    }
    let output = std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .ok()?;
    if output.status.success() {
        let t = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !t.is_empty() {
            return Some(t);
        }
    }
    None
}

fn github_client() -> Client {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(token) = github_token() {
        if let Ok(val) = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}")) {
            headers.insert(reqwest::header::AUTHORIZATION, val);
        }
    }
    Client::builder()
        .timeout(Duration::from_secs(GITHUB_TIMEOUT_SECS))
        .user_agent(format!("apl-cli/{}", env!("CARGO_PKG_VERSION")))
        .default_headers(headers)
        .build()
        .expect("failed to build HTTP client")
}

fn fetch_latest_version(client: &Client) -> Result<String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let resp = client.get(&url).send()?;
    if resp.status() == reqwest::StatusCode::FORBIDDEN
        || resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
    {
        bail!("GitHub API rate limit exceeded. Try again later or set GITHUB_TOKEN.");
    }
    let release: GithubRelease = resp.error_for_status()?.json()?;
    Ok(release.tag_name.trim_start_matches('v').to_string())
}

fn read_cache() -> Option<VersionCache> {
    let data = fs::read_to_string(cache_path()).ok()?;
    serde_json::from_str(&data).ok()
}

fn write_cache(ver: &str) {
    let cache = VersionCache {
        last_check: now_epoch(),
        latest_version: ver.to_string(),
    };
    if let Ok(json) = serde_json::to_string(&cache) {
        let _ = fs::create_dir_all(cache_dir());
        let _ = fs::write(cache_path(), json);
    }
}

fn parse_semver(v: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = v.trim_start_matches('v').splitn(3, '.').collect();
    if parts.len() == 3 {
        Some((parts[0].parse().ok()?, parts[1].parse().ok()?, parts[2].parse().ok()?))
    } else {
        None
    }
}

fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_semver(latest), parse_semver(current)) {
        (Some(l), Some(c)) => l > c,
        _ => latest != current,
    }
}

/// Print a one-line hint to stderr if a newer version is available.
/// Silently does nothing on any error (network, parse, etc.).
pub fn check_version_hint() {
    let _ = check_version_hint_inner();
}

fn check_version_hint_inner() -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");

    let latest = if let Some(cache) = read_cache() {
        if now_epoch() - cache.last_check < CHECK_INTERVAL_SECS {
            cache.latest_version
        } else {
            let client = github_client();
            let ver = fetch_latest_version(&client)?;
            write_cache(&ver);
            ver
        }
    } else {
        let client = github_client();
        let ver = fetch_latest_version(&client)?;
        write_cache(&ver);
        ver
    };

    if is_newer(&latest, current) {
        eprintln!(
            "\n{} {} -> {} (run {} to upgrade)",
            "New version available:".yellow(),
            current.dimmed(),
            latest.green().bold(),
            "apl upgrade".cyan().bold(),
        );
    }
    Ok(())
}

// ── upgrade command ─────────────────────────────────────────────

pub fn cmd_upgrade() -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    let client = github_client();

    println!("{}", "Checking for updates...".dimmed());
    let latest = fetch_latest_version(&client)
        .context("Failed to check latest version from GitHub")?;

    write_cache(&latest);

    if !is_newer(&latest, current) {
        println!(
            "{} You are already on the latest version ({}).",
            "Up to date.".green().bold(),
            current,
        );
        return Ok(());
    }

    println!(
        "  Current : {}\n  Latest  : {}",
        current.dimmed(),
        latest.green().bold(),
    );

    let target = detect_target()?;
    let url = format!(
        "https://github.com/{REPO}/releases/download/v{latest}/apl-{target}.tar.gz"
    );
    println!("  Target  : {target}");
    println!("{}", "Downloading...".dimmed());

    let bytes = client
        .get(&url)
        .timeout(Duration::from_secs(60))
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.bytes())
        .context("Failed to download release binary")?;

    let tmp = tempfile::tempdir().context("Failed to create temp dir")?;
    let tarball = tmp.path().join("apl.tar.gz");
    let mut f = fs::File::create(&tarball)?;
    f.write_all(&bytes)?;
    drop(f);

    let status = std::process::Command::new("tar")
        .args(["xzf", &tarball.to_string_lossy(), "-C", &tmp.path().to_string_lossy()])
        .status()
        .context("Failed to run tar")?;
    if !status.success() {
        bail!("tar extraction failed");
    }

    let new_bin = tmp.path().join("apl");
    if !new_bin.exists() {
        bail!("Extracted archive does not contain 'apl' binary");
    }

    let current_exe = env::current_exe().context("Cannot determine current executable path")?;
    let dest = fs::canonicalize(&current_exe)
        .unwrap_or(current_exe);

    let backup = dest.with_extension("old");
    if backup.exists() {
        let _ = fs::remove_file(&backup);
    }
    fs::rename(&dest, &backup)
        .with_context(|| format!("Failed to back up current binary ({}). Try with sudo?", dest.display()))?;

    if let Err(e) = fs::copy(&new_bin, &dest) {
        let _ = fs::rename(&backup, &dest);
        bail!("Failed to install new binary: {e}. Rolled back to previous version.");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(0o755));
    }

    let _ = fs::remove_file(&backup);

    println!(
        "\n{} apl {} -> {}",
        "Upgraded!".green().bold(),
        current,
        latest.green().bold(),
    );

    sync_skill(&client);
    Ok(())
}

// ── skill sync ──────────────────────────────────────────────────

fn sync_skill(client: &Client) {
    if let Err(e) = sync_skill_inner(client) {
        eprintln!("{} skill sync: {e}", "Warning:".yellow().bold());
    }
}

fn sync_skill_inner(client: &Client) -> Result<()> {
    let tree = fetch_repo_tree(client)?;
    let remote_skills = discover_remote_skills(&tree);

    if remote_skills.is_empty() {
        return Ok(());
    }

    for (skill_name, file_paths) in &remote_skills {
        let remote_skill_md_path = format!("{SKILLS_DIR}/{skill_name}/SKILL.md");
        let remote_content = client
            .get(&format!("{RAW_BASE}/{remote_skill_md_path}"))
            .timeout(Duration::from_secs(10))
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.text());
        let remote_content = match remote_content {
            Ok(c) => c,
            Err(_) => continue,
        };
        let remote_ver = match parse_skill_version(&remote_content) {
            Some(v) => v,
            None => continue,
        };

        let local_dirs = collect_skill_dirs(skill_name);
        for dir in &local_dirs {
            let local_ver = fs::read_to_string(dir.join("SKILL.md"))
                .ok()
                .and_then(|c| parse_skill_version(&c))
                .unwrap_or_else(|| "0.0.0".to_string());

            if !is_newer(&remote_ver, &local_ver) {
                continue;
            }

            let prefix = format!("{SKILLS_DIR}/{skill_name}/");
            for full_path in file_paths {
                let rel = full_path.strip_prefix(&prefix).unwrap_or(full_path);
                let raw_url = format!("{RAW_BASE}/{full_path}");
                let content = client
                    .get(&raw_url)
                    .timeout(Duration::from_secs(10))
                    .send()
                    .and_then(|r| r.error_for_status())
                    .and_then(|r| r.text());
                let content = match content {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let dest = dir.join(rel);
                if let Some(parent) = dest.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                fs::write(&dest, &content)
                    .with_context(|| format!("Failed to write {}", dest.display()))?;
            }

            println!(
                "{} {} ({} -> {})",
                "Skill updated:".cyan().bold(),
                dir.display(),
                local_ver.dimmed(),
                remote_ver.green(),
            );
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct TreeEntry {
    path: String,
    #[serde(rename = "type")]
    entry_type: String,
}

#[derive(Deserialize)]
struct TreeResponse {
    tree: Vec<TreeEntry>,
}

fn fetch_repo_tree(client: &Client) -> Result<TreeResponse> {
    let url = format!(
        "https://api.github.com/repos/{REPO}/git/trees/main?recursive=1"
    );
    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(10))
        .send()?;
    if !resp.status().is_success() {
        bail!("Failed to fetch repo tree: HTTP {}", resp.status());
    }
    Ok(resp.json()?)
}

/// Discover all skill directories under `skills/` from the repo tree.
/// Returns a map of skill_name -> list of file paths.
fn discover_remote_skills(tree: &TreeResponse) -> std::collections::HashMap<String, Vec<String>> {
    let mut skills: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let prefix = format!("{SKILLS_DIR}/");

    for entry in &tree.tree {
        if entry.entry_type != "blob" || !entry.path.starts_with(&prefix) {
            continue;
        }
        let rest = &entry.path[prefix.len()..];
        if let Some(slash) = rest.find('/') {
            let name = &rest[..slash];
            if rest.ends_with("AGENTS.md") {
                continue;
            }
            skills.entry(name.to_string()).or_default().push(entry.path.clone());
        }
    }

    skills.retain(|_, files| files.iter().any(|f| f.ends_with("/SKILL.md")));
    skills
}

fn parse_skill_version(content: &str) -> Option<String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let after_open = &trimmed[3..];
    let end = after_open.find("---")?;
    let frontmatter = &after_open[..end];
    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("version:") {
            return Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
        }
    }
    None
}

/// Find local directories matching a skill name (exact match).
/// Checks `~/.agents/skills/` and `./.agents/skills/`.
fn collect_skill_dirs(skill_name: &str) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let home = env::var("HOME").unwrap_or_default();
    let cwd = env::current_dir().unwrap_or_default();

    let search_roots: Vec<PathBuf> = vec![
        PathBuf::from(&home).join(".agents").join("skills"),
        cwd.join(".agents").join("skills"),
    ];

    for root in &search_roots {
        let entries = match fs::read_dir(root) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let dir_name = entry.file_name();
            if dir_name == skill_name {
                let path = entry.path();
                if path.join("SKILL.md").is_file() {
                    dirs.push(path);
                }
            }
        }
    }

    dirs.sort();
    dirs.dedup();
    dirs
}

fn detect_target() -> Result<&'static str> {
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    { Ok("x86_64-unknown-linux-gnu") }
    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    { Ok("aarch64-unknown-linux-gnu") }
    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    { Ok("x86_64-apple-darwin") }
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    { Ok("aarch64-apple-darwin") }
    #[cfg(not(any(
        all(target_arch = "x86_64", target_os = "linux"),
        all(target_arch = "aarch64", target_os = "linux"),
        all(target_arch = "x86_64", target_os = "macos"),
        all(target_arch = "aarch64", target_os = "macos"),
    )))]
    { bail!("Unsupported platform. Please install manually from https://github.com/{}", REPO) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.3.0", "0.2.0"));
        assert!(is_newer("0.2.1", "0.2.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(!is_newer("0.2.0", "0.2.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
    }

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver("0.2.0"), Some((0, 2, 0)));
        assert_eq!(parse_semver("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_semver("bad"), None);
    }

    #[test]
    fn test_parse_skill_version() {
        let content = "---\nname: apl-cli\nversion: 0.3.0\ndescription: test\n---\n# Hello";
        assert_eq!(parse_skill_version(content), Some("0.3.0".into()));

        let quoted = "---\nversion: \"1.2.3\"\n---\n";
        assert_eq!(parse_skill_version(quoted), Some("1.2.3".into()));

        assert_eq!(parse_skill_version("no frontmatter"), None);
    }

}
