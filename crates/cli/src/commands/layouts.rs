use anyhow::Result;
use serde_json::json;

use crate::api::client::qs;
use crate::api::types::Layout;
use crate::cli::{GlobalArgs, LayoutsSub};
use crate::output;

pub fn run(cmd: LayoutsSub, global: &GlobalArgs) -> Result<()> {
    match cmd {
        LayoutsSub::List => list(global),
        LayoutsSub::Add { name, active } => add(global, name, active),
        LayoutsSub::Duplicate { id } => duplicate(global, id),
        LayoutsSub::Rename { id, name } => rename(global, id, name),
        LayoutsSub::Delete {
            id,
            preserve_elements,
            preserve_layout_id,
        } => delete(global, id, preserve_elements, preserve_layout_id),
        LayoutsSub::SetActive { id, name } => set_active(global, id, name),
    }
}

fn list(global: &GlobalArgs) -> Result<()> {
    let response = super::client_for(global)?.get("layouts")?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    let layouts: Vec<Layout> = serde_json::from_value(response["layouts"].clone())?;
    let rows: Vec<Vec<String>> = layouts
        .iter()
        .map(|l| {
            vec![
                l.id.to_string(),
                l.name.clone(),
                if l.active {
                    "*".to_string()
                } else {
                    String::new()
                },
            ]
        })
        .collect();
    output::table(&["ID", "NAME", "ACTIVE"], &rows);
    Ok(())
}

fn add(global: &GlobalArgs, name: String, active: bool) -> Result<()> {
    let mut body = json!({ "name": name });
    if active {
        body["active"] = json!(true);
    }
    let response = super::client_for(global)?.post("layouts", body)?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    let layout = &response["layout"];
    println!(
        "Created layout {} '{}'",
        layout["id"],
        layout["name"].as_str().unwrap_or("")
    );
    Ok(())
}

fn duplicate(global: &GlobalArgs, id: u32) -> Result<()> {
    let response =
        super::client_for(global)?.post(&format!("layouts/duplicate?id={id}"), json!({}))?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    let layout = &response["layout"];
    println!(
        "Duplicated layout {id} -> {} '{}'",
        layout["id"],
        layout["name"].as_str().unwrap_or("")
    );
    Ok(())
}

fn rename(global: &GlobalArgs, id: u32, name: String) -> Result<()> {
    let response =
        super::client_for(global)?.patch(&format!("layouts?id={id}"), json!({ "name": name }))?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    println!("OK");
    Ok(())
}

fn delete(
    global: &GlobalArgs,
    id: u32,
    preserve_elements: bool,
    preserve_layout_id: Option<u32>,
) -> Result<()> {
    let query = qs(&[
        ("id", Some(id.to_string())),
        (
            "preserveElements",
            preserve_elements.then(|| "true".to_string()),
        ),
        (
            "preserveLayoutId",
            preserve_layout_id.map(|v| v.to_string()),
        ),
    ]);
    let response = super::client_for(global)?.delete(&format!("layouts{query}"))?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    println!("OK");
    Ok(())
}

fn set_active(global: &GlobalArgs, id: Option<u32>, name: Option<String>) -> Result<()> {
    let query = qs(&[("id", id.map(|v| v.to_string())), ("name", name)]);
    let response = super::client_for(global)?.post(&format!("layout{query}"), json!({}))?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    let layout = &response["activeLayout"];
    println!(
        "Active layout: {} '{}'",
        layout["id"],
        layout["name"].as_str().unwrap_or("")
    );
    Ok(())
}
