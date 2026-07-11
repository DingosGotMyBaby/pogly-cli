use anyhow::Result;
use serde_json::{Map, Value};

use crate::api::client::qs;
use crate::api::types::Folder;
use crate::cli::{FoldersSub, GlobalArgs};
use crate::output;

pub fn run(cmd: FoldersSub, global: &GlobalArgs) -> Result<()> {
    match cmd {
        FoldersSub::List => list(global),
        FoldersSub::Add { name, icon } => add(global, name, icon),
        FoldersSub::Update { id, name, icon } => update(global, id, name, icon),
        FoldersSub::Delete {
            id,
            no_preserve_elements,
        } => delete(global, id, no_preserve_elements),
    }
}

fn list(global: &GlobalArgs) -> Result<()> {
    let response = super::client_for(global)?.get("folders")?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    let folders: Vec<Folder> = serde_json::from_value(response["folders"].clone())?;
    if folders.is_empty() {
        println!("No folders found.");
        return Ok(());
    }
    let rows: Vec<Vec<String>> = folders
        .iter()
        .map(|f| {
            vec![
                f.id.to_string(),
                f.name.clone(),
                f.icon.clone(),
                f.created_by.clone(),
            ]
        })
        .collect();
    output::table(&["ID", "NAME", "ICON", "CREATED-BY"], &rows);
    Ok(())
}

fn add(global: &GlobalArgs, name: String, icon: Option<String>) -> Result<()> {
    let mut body = Map::new();
    body.insert("name".into(), name.into());
    if let Some(icon) = icon {
        body.insert("icon".into(), icon.into());
    }
    let response = super::client_for(global)?.post("folders", Value::Object(body))?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    let folder = &response["folder"];
    println!(
        "Created folder {} '{}'",
        folder["id"],
        folder["name"].as_str().unwrap_or("")
    );
    Ok(())
}

fn update(global: &GlobalArgs, id: u32, name: Option<String>, icon: Option<String>) -> Result<()> {
    let mut body = Map::new();
    if let Some(name) = name {
        body.insert("name".into(), name.into());
    }
    if let Some(icon) = icon {
        body.insert("icon".into(), icon.into());
    }
    if body.is_empty() {
        anyhow::bail!("provide --name and/or --icon");
    }
    let response =
        super::client_for(global)?.patch(&format!("folders?id={id}"), Value::Object(body))?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    println!("OK");
    Ok(())
}

fn delete(global: &GlobalArgs, id: u32, no_preserve_elements: bool) -> Result<()> {
    let query = qs(&[
        ("id", Some(id.to_string())),
        (
            "preserveElements",
            no_preserve_elements.then(|| "false".to_string()),
        ),
    ]);
    let response = super::client_for(global)?.delete(&format!("folders{query}"))?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    println!("OK");
    Ok(())
}
