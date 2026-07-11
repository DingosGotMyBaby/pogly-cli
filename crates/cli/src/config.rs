use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths;

pub const DEFAULT_HOST: &str = "https://maincloud.spacetimedb.com";

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub default_overlay: Option<String>,
    #[serde(default)]
    pub overlays: Vec<OverlayProfile>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OverlayProfile {
    pub nickname: String,
    pub module: String,
    pub token: String,
    pub host: String,
}

impl Config {
    pub fn load() -> Result<Config> {
        let path = paths::config_file();
        if !path.is_file() {
            return Ok(Config::default());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        toml::from_str(&raw).with_context(|| format!("invalid config at {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = paths::config_file();
        std::fs::create_dir_all(paths::config_dir())?;
        std::fs::write(&path, toml::to_string_pretty(self)?)
            .with_context(|| format!("failed to write {}", path.display()))
    }

    pub fn find(&self, nickname: &str) -> Option<&OverlayProfile> {
        self.overlays.iter().find(|o| o.nickname == nickname)
    }

    pub fn find_mut(&mut self, nickname: &str) -> Option<&mut OverlayProfile> {
        self.overlays.iter_mut().find(|o| o.nickname == nickname)
    }

    pub fn resolve(&self, overlay_flag: Option<&str>) -> Result<&OverlayProfile> {
        if let Some(nick) = overlay_flag {
            return self
                .find(nick)
                .with_context(|| format!("no overlay profile named '{nick}'"));
        }
        if let Some(default) = &self.default_overlay {
            if let Some(profile) = self.find(default) {
                return Ok(profile);
            }
        }
        match self.overlays.len() {
            0 => bail!("no overlays configured; run `pogly overlay add <identity-or-url> --token <pgly_...>`"),
            1 => Ok(&self.overlays[0]),
            _ => bail!("no default overlay set; run `pogly overlay set-default <nickname>`"),
        }
    }
}

pub struct ParsedTarget {
    pub module: String,
    pub host: Option<String>,
}

pub fn parse_overlay_target(input: &str) -> Result<ParsedTarget> {
    let input = input.trim();
    if input.contains("://") || input.contains("/overlay") {
        parse_overlay_url(input)
    } else if input.is_empty() {
        bail!("empty overlay target")
    } else {
        Ok(ParsedTarget {
            module: normalize_module(input),
            host: None,
        })
    }
}

fn parse_overlay_url(url: &str) -> Result<ParsedTarget> {
    let query = url
        .split_once('?')
        .map(|(_, q)| q)
        .context("overlay URL has no query string; expected ...?module=<identity>")?;
    let mut module = None;
    let mut domain = None;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        match percent_decode(key).to_ascii_lowercase().as_str() {
            "module" => module = Some(percent_decode(value)),
            "domain" => domain = Some(percent_decode(value)),
            _ => {}
        }
    }
    let module = module
        .filter(|m| !m.is_empty())
        .context("overlay URL has no module parameter")?;
    Ok(ParsedTarget {
        module: normalize_module(&module),
        host: domain.filter(|d| !d.is_empty()).map(|d| normalize_host(&d)),
    })
}

pub fn is_identity_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

// Mirrors the frontend: 64-hex identities pass through, anything else is a
// legacy database name that gets the "pogly-" prefix re-added.
pub fn normalize_module(value: &str) -> String {
    let value = value.trim();
    if is_identity_hex(value) {
        value.to_ascii_lowercase()
    } else if value.starts_with("pogly-") {
        value.to_string()
    } else {
        format!("pogly-{}", value.replace('_', "-").to_lowercase())
    }
}

// Shared overlay URLs carry the SpacetimeDB host as a websocket origin.
pub fn normalize_host(host: &str) -> String {
    let host = host.trim().trim_end_matches('/');
    if let Some(rest) = host.strip_prefix("wss://") {
        format!("https://{rest}")
    } else if let Some(rest) = host.strip_prefix("ws://") {
        format!("http://{rest}")
    } else if host.contains("://") {
        host.to_string()
    } else {
        format!("https://{host}")
    }
}

pub fn short_module(module: &str) -> String {
    if is_identity_hex(module) {
        format!("{}…", &module[..8])
    } else {
        module.to_string()
    }
}

pub fn short_token(token: &str) -> String {
    if token.len() > 4 {
        format!("pgly_…{}", &token[token.len() - 4..])
    } else {
        "pgly_…".to_string()
    }
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
                match hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                    Some(b) => {
                        out.push(b);
                        i += 3;
                    }
                    None => {
                        out.push(bytes[i]);
                        i += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDENTITY: &str = "c200a94ce78ba0b0f4a121022g00000000000000000000000000000000000000";
    const HEX_IDENTITY: &str = "c200a94ce78ba0b0f4a1210220abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn identity_hex_detection() {
        assert!(is_identity_hex(HEX_IDENTITY));
        assert!(is_identity_hex(&HEX_IDENTITY.to_uppercase()));
        assert!(!is_identity_hex(IDENTITY));
        assert!(!is_identity_hex("pogly-mystream"));
        assert!(!is_identity_hex(&HEX_IDENTITY[..63]));
    }

    #[test]
    fn bare_identity_passes_through_lowercased() {
        let parsed = parse_overlay_target(&HEX_IDENTITY.to_uppercase()).unwrap();
        assert_eq!(parsed.module, HEX_IDENTITY);
        assert!(parsed.host.is_none());
    }

    #[test]
    fn legacy_name_gets_prefix_and_normalization() {
        assert_eq!(normalize_module("My_Stream"), "pogly-my-stream");
        assert_eq!(normalize_module("pogly-already"), "pogly-already");
    }

    #[test]
    fn url_with_identity() {
        let parsed = parse_overlay_target(&format!(
            "https://cloud.pogly.gg/overlay?module={HEX_IDENTITY}"
        ))
        .unwrap();
        assert_eq!(parsed.module, HEX_IDENTITY);
        assert!(parsed.host.is_none());
    }

    #[test]
    fn url_with_legacy_name_and_domain() {
        let parsed = parse_overlay_target(
            "https://selfhost.example/overlay?module=mystream&domain=wss%3A%2F%2Fstdb.example.com",
        )
        .unwrap();
        assert_eq!(parsed.module, "pogly-mystream");
        assert_eq!(parsed.host.as_deref(), Some("https://stdb.example.com"));
    }

    #[test]
    fn ws_domain_normalizes_to_http() {
        assert_eq!(
            normalize_host("ws://localhost:3000/"),
            "http://localhost:3000"
        );
        assert_eq!(
            normalize_host("stdb.example.com"),
            "https://stdb.example.com"
        );
        assert_eq!(
            normalize_host("https://stdb.example.com"),
            "https://stdb.example.com"
        );
    }

    #[test]
    fn url_without_module_errors() {
        assert!(parse_overlay_target("https://cloud.pogly.gg/overlay?other=1").is_err());
        assert!(parse_overlay_target("https://cloud.pogly.gg/overlay").is_err());
    }
}
