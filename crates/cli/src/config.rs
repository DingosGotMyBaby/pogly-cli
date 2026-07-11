use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths;

pub const HOST: &str = "https://maincloud.spacetimedb.com";

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub default_overlay: Option<String>,
    #[serde(default)]
    pub overlays: Vec<OverlayProfile>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OverlayProfile {
    pub nickname: String,
    #[serde(alias = "module")]
    pub address: String,
    pub token: String,
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
            0 => bail!("no overlays configured; run `pogly overlay add <address-or-url> --token <pgly_...>`"),
            1 => Ok(&self.overlays[0]),
            _ => bail!("no default overlay set; run `pogly overlay set-default <nickname>`"),
        }
    }
}

pub fn parse_overlay_target(input: &str) -> Result<String> {
    let input = input.trim();
    if input.contains("://") || input.contains("/overlay") {
        parse_overlay_url(input)
    } else if input.is_empty() {
        bail!("empty overlay target")
    } else {
        Ok(normalize_address(input))
    }
}

fn parse_overlay_url(url: &str) -> Result<String> {
    let query = url
        .split_once('?')
        .map(|(_, q)| q)
        .context("overlay URL has no query string; expected ...?module=<overlay address>")?;
    let address = query
        .split('&')
        .filter_map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            (percent_decode(key).eq_ignore_ascii_case("module")).then(|| percent_decode(value))
        })
        .find(|v| !v.is_empty())
        .context("overlay URL has no overlay address (?module=...) parameter")?;
    Ok(normalize_address(&address))
}

pub fn is_identity_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

// Mirrors the frontend: 64-hex addresses pass through, anything else is a
// legacy database name that gets the "pogly-" prefix re-added.
pub fn normalize_address(value: &str) -> String {
    let value = value.trim();
    if is_identity_hex(value) {
        value.to_ascii_lowercase()
    } else if value.starts_with("pogly-") {
        value.to_string()
    } else {
        format!("pogly-{}", value.replace('_', "-").to_lowercase())
    }
}

pub fn short_address(address: &str) -> String {
    if is_identity_hex(address) {
        format!("{}…", &address[..8])
    } else {
        address.to_string()
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

    const NOT_HEX: &str = "c200a94ce78ba0b0f4a121022g00000000000000000000000000000000000000";
    const HEX_ADDRESS: &str = "c200a94ce78ba0b0f4a1210220abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn identity_hex_detection() {
        assert!(is_identity_hex(HEX_ADDRESS));
        assert!(is_identity_hex(&HEX_ADDRESS.to_uppercase()));
        assert!(!is_identity_hex(NOT_HEX));
        assert!(!is_identity_hex("pogly-mystream"));
        assert!(!is_identity_hex(&HEX_ADDRESS[..63]));
    }

    #[test]
    fn bare_address_passes_through_lowercased() {
        let address = parse_overlay_target(&HEX_ADDRESS.to_uppercase()).unwrap();
        assert_eq!(address, HEX_ADDRESS);
    }

    #[test]
    fn legacy_name_gets_prefix_and_normalization() {
        assert_eq!(normalize_address("My_Stream"), "pogly-my-stream");
        assert_eq!(normalize_address("pogly-already"), "pogly-already");
    }

    #[test]
    fn url_with_address() {
        let address = parse_overlay_target(&format!(
            "https://cloud.pogly.gg/overlay?module={HEX_ADDRESS}"
        ))
        .unwrap();
        assert_eq!(address, HEX_ADDRESS);
    }

    #[test]
    fn url_with_legacy_name_and_extra_params() {
        let address = parse_overlay_target(
            "https://cloud.pogly.gg/overlay?module=mystream&domain=wss%3A%2F%2Fignored.example",
        )
        .unwrap();
        assert_eq!(address, "pogly-mystream");
    }

    #[test]
    fn url_without_address_errors() {
        assert!(parse_overlay_target("https://cloud.pogly.gg/overlay?other=1").is_err());
        assert!(parse_overlay_target("https://cloud.pogly.gg/overlay").is_err());
    }

    #[test]
    fn old_config_module_key_still_loads() {
        let config: Config = toml::from_str(
            "default_overlay = \"main\"\n[[overlays]]\nnickname = \"main\"\nmodule = \"pogly-old\"\ntoken = \"pgly_x\"\nhost = \"https://ignored.example\"\n",
        )
        .unwrap();
        assert_eq!(config.overlays[0].address, "pogly-old");
    }
}
