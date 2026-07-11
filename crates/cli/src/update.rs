use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ureq::Agent;

use crate::paths;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    #[serde(default)]
    pub update_check: UpdateCheck,
}

#[derive(Serialize, Deserialize, Default)]
pub struct UpdateCheck {
    pub last_checked_unix: Option<u64>,
    pub latest_version: Option<String>,
}

pub fn releases_repo() -> String {
    std::env::var("POGLY_RELEASES_REPO").unwrap_or_else(|_| "PoglyApp/pogly-cli".to_string())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn load_state() -> State {
    std::fs::read_to_string(paths::state_file())
        .ok()
        .and_then(|raw| toml::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_state(state: &State) {
    let _ = std::fs::create_dir_all(paths::local_dir());
    if let Ok(raw) = toml::to_string_pretty(state) {
        let _ = std::fs::write(paths::state_file(), raw);
    }
}

// Attempt timestamps are stamped even on failure so an offline machine pays
// the (short) network timeout at most once per day.
pub fn passive_check() {
    let mut state = load_state();
    let now = unix_now();
    if state
        .update_check
        .last_checked_unix
        .is_some_and(|t| now.saturating_sub(t) < 24 * 3600)
    {
        return;
    }
    state.update_check.last_checked_unix = Some(now);
    if let Ok(release) = fetch_release("latest", Duration::from_millis(1500)) {
        if let Some(version) = release_version(&release) {
            state.update_check.latest_version = Some(version);
        }
    }
    save_state(&state);
}

pub fn cached_latest() -> Option<String> {
    load_state().update_check.latest_version
}

pub fn update_available() -> Option<(String, String)> {
    let latest = cached_latest()?;
    let newer = matches!(
        (semver::Version::parse(&latest), semver::Version::parse(CURRENT_VERSION)),
        (Ok(l), Ok(c)) if l > c
    );
    newer.then(|| (CURRENT_VERSION.to_string(), latest))
}

pub fn release_version(release: &Value) -> Option<String> {
    release
        .get("tag_name")
        .and_then(Value::as_str)
        .map(|t| t.trim_start_matches('v').to_string())
}

pub fn fetch_release(tag: &str, timeout: Duration) -> Result<Value> {
    let path = if tag == "latest" {
        "latest".to_string()
    } else {
        format!("tags/v{}", tag.trim_start_matches('v'))
    };
    let url = format!(
        "https://api.github.com/repos/{}/releases/{path}",
        releases_repo()
    );
    let config = Agent::config_builder()
        .timeout_global(Some(timeout))
        .user_agent(concat!("pogly-cli/", env!("CARGO_PKG_VERSION")))
        .build();
    let mut response = config
        .new_agent()
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .call()
        .with_context(|| format!("failed to fetch release info from {url}"))?;
    let body = response
        .body_mut()
        .read_to_string()
        .context("failed to read release info")?;
    serde_json::from_str(&body).context("invalid release info")
}
