use anyhow::Result;

use crate::api::types::WhoAmI;
use crate::cli::GlobalArgs;
use crate::output;

pub fn run(global: &GlobalArgs) -> Result<()> {
    let response = super::client_for(global)?.get("whoami")?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    let who: WhoAmI = serde_json::from_value(response)?;
    print_whoami(&who);
    Ok(())
}

pub fn print_whoami(who: &WhoAmI) {
    println!("Token:       {} (id {})", who.label, who.token_id);
    println!("Read-only:   {}", who.read_only);
    println!("Identity:    {}", who.identity);
    println!("Permissions: {}", who.permissions.join(", "));
}
