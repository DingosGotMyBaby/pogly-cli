use anyhow::Result;
use serde_json::{Map, Value};

use crate::api::client::qs;
use crate::api::types::ElementData;
use crate::cli::{DataType, ElementDataSub, GlobalArgs};
use crate::output;

pub fn run(cmd: ElementDataSub, global: &GlobalArgs) -> Result<()> {
    match cmd {
        ElementDataSub::List { id, name, folder } => list(global, id, name, folder),
        ElementDataSub::Add {
            name,
            r#type,
            data,
            width,
            height,
            folder_id,
        } => add(global, name, r#type, data, width, height, folder_id),
        ElementDataSub::Update {
            id,
            name,
            data,
            width,
            height,
            folder_id,
        } => update(global, id, name, data, width, height, folder_id),
        ElementDataSub::Delete { id, name } => delete(global, id, name),
    }
}

fn list(
    global: &GlobalArgs,
    id: Option<u32>,
    name: Option<String>,
    folder: Option<u32>,
) -> Result<()> {
    let query = qs(&[
        ("id", id.map(|v| v.to_string())),
        ("name", name),
        ("folder", folder.map(|v| v.to_string())),
    ]);
    let response = super::client_for(global)?.get(&format!("elementdata{query}"))?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    let assets: Vec<ElementData> = serde_json::from_value(response["elementData"].clone())?;
    if assets.is_empty() {
        println!("No element data found.");
        return Ok(());
    }
    let rows: Vec<Vec<String>> = assets
        .iter()
        .map(|a| {
            vec![
                a.id.to_string(),
                output::truncate(&a.name, 30),
                a.kind.clone(),
                format!("{}x{}", a.width, a.height),
                a.folder_id.to_string(),
                a.created_by.clone(),
            ]
        })
        .collect();
    output::table(
        &["ID", "NAME", "TYPE", "SIZE", "FOLDER", "CREATED-BY"],
        &rows,
    );
    Ok(())
}

fn add(
    global: &GlobalArgs,
    name: String,
    kind: DataType,
    data: String,
    width: i64,
    height: i64,
    folder_id: Option<u32>,
) -> Result<()> {
    let mut body = Map::new();
    body.insert("name".into(), name.into());
    body.insert("type".into(), kind.as_str().into());
    body.insert("data".into(), data.into());
    body.insert("width".into(), width.into());
    body.insert("height".into(), height.into());
    if let Some(folder_id) = folder_id {
        body.insert("folderId".into(), folder_id.into());
    }
    let response = super::client_for(global)?.post("elementdata", Value::Object(body))?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    let asset = &response["elementData"];
    println!(
        "Created element data {} '{}' ({} {}x{})",
        asset["id"],
        asset["name"].as_str().unwrap_or(""),
        asset["type"].as_str().unwrap_or("?"),
        asset["width"],
        asset["height"]
    );
    Ok(())
}

fn update(
    global: &GlobalArgs,
    id: u32,
    name: Option<String>,
    data: Option<String>,
    width: Option<i64>,
    height: Option<i64>,
    folder_id: Option<u32>,
) -> Result<()> {
    let mut body = Map::new();
    if let Some(name) = name {
        body.insert("name".into(), name.into());
    }
    if let Some(data) = data {
        body.insert("data".into(), data.into());
    }
    if let Some(width) = width {
        body.insert("width".into(), width.into());
    }
    if let Some(height) = height {
        body.insert("height".into(), height.into());
    }
    if let Some(folder_id) = folder_id {
        body.insert("folderId".into(), folder_id.into());
    }
    if body.is_empty() {
        anyhow::bail!("provide at least one of --name, --data, --width, --height, --folder-id");
    }
    let response =
        super::client_for(global)?.patch(&format!("elementdata?id={id}"), Value::Object(body))?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    println!("OK");
    Ok(())
}

fn delete(global: &GlobalArgs, id: Option<u32>, name: Option<String>) -> Result<()> {
    let query = qs(&[("id", id.map(|v| v.to_string())), ("name", name)]);
    let response = super::client_for(global)?.delete(&format!("elementdata{query}"))?;
    if global.json {
        output::print_json(&response);
        return Ok(());
    }
    println!("OK");
    Ok(())
}
