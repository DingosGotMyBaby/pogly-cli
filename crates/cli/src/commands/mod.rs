use anyhow::Result;

use crate::api::client::ApiClient;
use crate::cli::{Cli, Cmd, GlobalArgs};
use crate::config::Config;

mod elementdata;
mod elements;
mod folders;
mod layouts;
mod overlay;
mod ping;
mod version;
mod whoami;

pub fn run(cli: Cli) -> Result<()> {
    let global = cli.global;
    match cli.cmd {
        Cmd::Ping => ping::run(&global),
        Cmd::Whoami => whoami::run(&global),
        Cmd::Overlay(c) => overlay::run(c.cmd, &global),
        Cmd::Elements(c) => elements::run(c.cmd, &global),
        Cmd::Elementdata(c) => elementdata::run(c.cmd, &global),
        Cmd::Layouts(c) => layouts::run(c.cmd, &global),
        Cmd::Folders(c) => folders::run(c.cmd, &global),
        Cmd::Version(c) => version::run(c),
    }
}

pub fn client_for(global: &GlobalArgs) -> Result<ApiClient> {
    let config = Config::load()?;
    let profile = config.resolve(global.overlay.as_deref())?;
    let host = global.host.as_deref().unwrap_or(&profile.host);
    Ok(ApiClient::new(host, &profile.module, &profile.token))
}
