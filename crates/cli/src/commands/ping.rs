use anyhow::Result;

use crate::cli::GlobalArgs;
use crate::output;

pub fn run(global: &GlobalArgs) -> Result<()> {
    let response = super::client_for(global)?.get("ping")?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    let api = response["api"].as_str().unwrap_or("?");
    println!("Pogly API reachable (api {api})");
    Ok(())
}
