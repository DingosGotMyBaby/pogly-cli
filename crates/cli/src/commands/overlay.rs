use anyhow::{bail, Context, Result};

use crate::api::client::ApiClient;
use crate::api::types::WhoAmI;
use crate::cli::{GlobalArgs, OverlaySub};
use crate::config::{self, Config, OverlayProfile};
use crate::output;

pub fn run(cmd: OverlaySub, _global: &GlobalArgs) -> Result<()> {
    match cmd {
        OverlaySub::List => list(),
        OverlaySub::Add {
            target,
            token,
            nickname,
            no_verify,
        } => add(target, token, nickname, no_verify),
        OverlaySub::Update {
            nickname,
            token,
            address,
            rename,
        } => update(nickname, token, address, rename),
        OverlaySub::Remove { nickname } => remove(nickname),
        OverlaySub::SetDefault { nickname } => set_default(nickname),
    }
}

fn default_nickname(address: &str) -> String {
    if config::is_identity_hex(address) {
        address[..8].to_string()
    } else {
        address
            .strip_prefix("pogly-")
            .unwrap_or(address)
            .to_string()
    }
}

fn list() -> Result<()> {
    let config = Config::load()?;
    if config.overlays.is_empty() {
        println!("No overlays configured. Add one with `pogly overlay add <address-or-url> --token <pgly_...>`");
        return Ok(());
    }
    let rows: Vec<Vec<String>> = config
        .overlays
        .iter()
        .map(|o| {
            vec![
                if config.default_overlay.as_deref() == Some(&o.nickname) {
                    "*".to_string()
                } else {
                    String::new()
                },
                o.nickname.clone(),
                config::short_address(&o.address),
                config::short_token(&o.token),
            ]
        })
        .collect();
    output::table(&["", "NICKNAME", "OVERLAY ADDRESS", "TOKEN"], &rows);
    Ok(())
}

fn add(target: String, token: String, nickname: Option<String>, no_verify: bool) -> Result<()> {
    let address = config::parse_overlay_target(&target)?;
    let nickname = nickname.unwrap_or_else(|| default_nickname(&address));

    let mut config = Config::load()?;
    if config.find(&nickname).is_some() {
        bail!("an overlay named '{nickname}' already exists");
    }

    if !no_verify {
        let client = ApiClient::new(&address, &token);
        let response = client
            .get("whoami")
            .context("token validation failed (pass --no-verify to add anyway)")?;
        let who: WhoAmI = serde_json::from_value(response)?;
        super::whoami::print_whoami(&who);
        println!();
    }

    let first = config.overlays.is_empty();
    config.overlays.push(OverlayProfile {
        nickname: nickname.clone(),
        address,
        token,
    });
    if first {
        config.default_overlay = Some(nickname.clone());
    }
    config.save()?;
    println!(
        "Added overlay '{nickname}'{}",
        if first { " (default)" } else { "" }
    );
    Ok(())
}

fn update(
    nickname: String,
    token: Option<String>,
    address: Option<String>,
    rename: Option<String>,
) -> Result<()> {
    if token.is_none() && address.is_none() && rename.is_none() {
        bail!("provide at least one of --token, --address, --rename");
    }
    let mut config = Config::load()?;
    if let Some(new_name) = &rename {
        if new_name != &nickname && config.find(new_name).is_some() {
            bail!("an overlay named '{new_name}' already exists");
        }
    }
    let profile = config
        .find_mut(&nickname)
        .with_context(|| format!("no overlay profile named '{nickname}'"))?;
    if let Some(token) = token {
        profile.token = token;
    }
    if let Some(address) = address {
        profile.address = config::parse_overlay_target(&address)?;
    }
    if let Some(new_name) = rename {
        profile.nickname = new_name.clone();
        if config.default_overlay.as_deref() == Some(nickname.as_str()) {
            config.default_overlay = Some(new_name);
        }
    }
    config.save()?;
    println!("Updated overlay '{nickname}'");
    Ok(())
}

fn remove(nickname: String) -> Result<()> {
    let mut config = Config::load()?;
    if config.find(&nickname).is_none() {
        bail!("no overlay profile named '{nickname}'");
    }
    config.overlays.retain(|o| o.nickname != nickname);
    if config.default_overlay.as_deref() == Some(nickname.as_str()) {
        config.default_overlay = if config.overlays.len() == 1 {
            Some(config.overlays[0].nickname.clone())
        } else {
            None
        };
    }
    config.save()?;
    println!("Removed overlay '{nickname}'");
    Ok(())
}

fn set_default(nickname: String) -> Result<()> {
    let mut config = Config::load()?;
    if config.find(&nickname).is_none() {
        bail!("no overlay profile named '{nickname}'");
    }
    config.default_overlay = Some(nickname.clone());
    config.save()?;
    println!("Default overlay is now '{nickname}'");
    Ok(())
}
