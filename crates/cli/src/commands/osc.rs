use std::net::UdpSocket;
use anyhow::{bail, Result};
use serde_json::json;
use serde_json::Value;

use rosc::{OscPacket, OscType, OscMessage};
use crate::api::client::{qs, ApiClient};
use crate::cli::{GlobalArgs, OscCmd};

pub fn run(cmd: OscCmd, global: &GlobalArgs) -> Result<()> {
    let host = &cmd.host;
    let port = cmd.port;
    let addr = format!("{host}:{port}");

    let socket = UdpSocket::bind(&addr)?;
    println!("OSC listener running on {addr}");
    println!("Press Ctrl+C to stop.");

    let client = super::client_for(global)?;

    let mut buf = [0u8; 65535];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, src)) => {
                match rosc::decoder::decode_udp(&buf[..size]) {
                    Ok((_, packet)) => {
                        if let Err(e) = handle_packet(packet, &client) {
                            eprintln!("Error handling packet from {src}: {e:#}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error decoding OSC packet from {src}: {e:#}");
                    }
                }
            }
            Err(e) => {
                eprintln!("Socket receive error: {e}");
            }
        }
    }
}

fn handle_packet(packet: OscPacket, client: &ApiClient) -> Result<()> {
    match packet {
        OscPacket::Message(msg) => handle_message(msg, client),
        OscPacket::Bundle(bundle) => {
            for inner in bundle.content {
                if let Err(e) = handle_packet(inner, client) {
                    eprintln!("Error handling message in bundle: {e:#}");
                }
            }
            Ok(())
        }
    }
}

fn coerce_to_string(arg: &OscType) -> Result<String> {
    match arg {
        OscType::String(s) => Ok(s.clone()),
        OscType::Int(i) => Ok(i.to_string()),
        OscType::Long(l) => Ok(l.to_string()),
        OscType::Float(f) => Ok(f.to_string()),
        OscType::Double(d) => Ok(d.to_string()),
        OscType::Bool(b) => Ok(b.to_string()),
        _ => bail!("Cannot coerce OSC arg {:?} to string", arg),
    }
}

fn coerce_to_u32(arg: &OscType) -> Result<u32> {
    match arg {
        OscType::Int(i) => Ok(*i as u32),
        OscType::Long(l) => Ok(*l as u32),
        OscType::Float(f) => Ok(*f as u32),
        OscType::Double(d) => Ok(*d as u32),
        OscType::String(s) => s.parse::<u32>().map_err(|e| anyhow::anyhow!("Failed to parse string to u32: {e}")),
        _ => bail!("Cannot coerce OSC arg {:?} to u32", arg),
    }
}

fn coerce_to_i64(arg: &OscType) -> Result<i64> {
    match arg {
        OscType::Int(i) => Ok(*i as i64),
        OscType::Long(l) => Ok(*l),
        OscType::Float(f) => Ok(*f as i64),
        OscType::Double(d) => Ok(*d as i64),
        OscType::String(s) => s.parse::<i64>().map_err(|e| anyhow::anyhow!("Failed to parse string to i64: {e}")),
        _ => bail!("Cannot coerce OSC arg {:?} to i64", arg),
    }
}

fn coerce_to_bool(arg: &OscType) -> Result<bool> {
    match arg {
        OscType::Bool(b) => Ok(*b),
        OscType::Int(i) => Ok(*i != 0),
        OscType::Long(l) => Ok(*l != 0),
        OscType::Float(f) => Ok(*f != 0.0),
        OscType::Double(d) => Ok(*d != 0.0),
        OscType::String(s) => {
            let s_lower = s.to_lowercase();
            if s_lower == "true" || s_lower == "1" {
                Ok(true)
            } else if s_lower == "false" || s_lower == "0" {
                Ok(false)
            } else {
                bail!("Invalid boolean string: {s}")
            }
        }
        _ => bail!("Cannot coerce OSC arg {:?} to bool", arg),
    }
}

fn handle_message(msg: OscMessage, client: &ApiClient) -> Result<()> {
    let addr = msg.addr.as_str();
    println!("Received: {} ({} args)", addr, msg.args.len());

    match addr {
        "/pogly/ping" | "/ping" => {
            let response = client.get("ping")?;
            println!("Ping response: {response}");
        }
        "/pogly/whoami" | "/whoami" => {
            let response = client.get("whoami")?;
            println!("Whoami response: {response}");
        }
        "/pogly/layouts/set-active" | "/pogly/layouts/set_active" | "/pogly/active_layout" => {
            if msg.args.is_empty() {
                bail!("Expected 1 argument (layout name or ID)");
            }
            let arg = &msg.args[0];
            let response = if let Ok(id) = coerce_to_u32(arg) {
                let query = qs(&[("id", Some(id.to_string())), ("name", None)]);
                client.post(&format!("layout{query}"), json!({}))?
            } else {
                let name = coerce_to_string(arg)?;
                let query = qs(&[("id", None), ("name", Some(name))]);
                client.post(&format!("layout{query}"), json!({}))?
            };
            let layout = &response["activeLayout"];
            println!(
                "Switched to layout: {} '{}'",
                layout["id"],
                layout["name"].as_str().unwrap_or("")
            );
        }
        "/pogly/elements/delete" => {
            if msg.args.is_empty() {
                bail!("Expected 1 argument (element ID)");
            }
            let id = coerce_to_u32(&msg.args[0])?;
            client.delete(&format!("elements?id={id}"))?;
            println!("Deleted element {id}");
        }
        "/pogly/elements/update/position" => {
            if msg.args.len() < 3 {
                bail!("Expected 3 arguments (id, x, y)");
            }
            let id = coerce_to_u32(&msg.args[0])?;
            let x = coerce_to_i64(&msg.args[1])?;
            let y = coerce_to_i64(&msg.args[2])?;

            let body = json!({
                "x": x,
                "y": y
            });
            client.patch(&format!("elements?id={id}"), body)?;
            println!("Updated element {id} position to ({x}, {y})");
        }
        "/pogly/elements/update/transparency" => {
            if msg.args.len() < 2 {
                bail!("Expected 2 arguments (id, transparency)");
            }
            let id = coerce_to_u32(&msg.args[0])?;
            let transparency = coerce_to_i64(&msg.args[1])?;

            let body = json!({
                "transparency": transparency
            });
            client.patch(&format!("elements?id={id}"), body)?;
            println!("Updated element {id} transparency to {transparency}%");
        }
        "/pogly/elements/update/text" => {
            if msg.args.len() < 2 {
                bail!("Expected 2 arguments (id, text)");
            }
            let id = coerce_to_u32(&msg.args[0])?;
            let text = coerce_to_string(&msg.args[1])?;

            let body = json!({
                "text": {
                    "text": text
                }
            });
            client.patch(&format!("elements?id={id}"), body)?;
            println!("Updated element {id} text");
        }
        "/pogly/elements/update/media/playing" => {
            if msg.args.len() < 2 {
                bail!("Expected 2 arguments (id, playing)");
            }
            let id = coerce_to_u32(&msg.args[0])?;
            let playing = coerce_to_bool(&msg.args[1])?;

            let body = json!({
                "media": {
                    "playing": playing
                }
            });
            client.patch(&format!("elements?id={id}"), body)?;
            println!("Updated element {id} playing to {playing}");
        }
        "/pogly/elements/update/media/volume" => {
            if msg.args.len() < 2 {
                bail!("Expected 2 arguments (id, volume)");
            }
            let id = coerce_to_u32(&msg.args[0])?;
            let volume = coerce_to_i64(&msg.args[1])?;

            let body = json!({
                "media": {
                    "volume": volume
                }
            });
            client.patch(&format!("elements?id={id}"), body)?;
            println!("Updated element {id} volume to {volume}%");
        }
        "/pogly/elements/update/media/timestamp" => {
            if msg.args.len() < 2 {
                bail!("Expected 2 arguments (id, timestamp)");
            }
            let id = coerce_to_u32(&msg.args[0])?;
            let timestamp = coerce_to_i64(&msg.args[1])?;

            let body = json!({
                "media": {
                    "timestamp": timestamp
                }
            });
            client.patch(&format!("elements?id={id}"), body)?;
            println!("Updated element {id} timestamp to {timestamp}s");
        }
        "/pogly/elements/update" => {
            if msg.args.len() < 3 {
                bail!("Expected 3 arguments (id, field_name, value)");
            }
            let id = coerce_to_u32(&msg.args[0])?;
            let field_name = coerce_to_string(&msg.args[1])?;
            let raw_value = &msg.args[2];

            let body = build_generic_update_body(&field_name, raw_value)?;
            client.patch(&format!("elements?id={id}"), body)?;
            println!("Updated element {id} field '{field_name}'");
        }
        _ => {
            println!("Unknown OSC address: {addr}");
        }
    }

    Ok(())
}

fn build_generic_update_body(field_name: &str, raw_value: &OscType) -> Result<Value> {
    let mut body = serde_json::Map::new();

    match field_name {
        // Root fields
        "x" | "y" | "transparency" | "layout_id" | "layoutId" => {
            let val = coerce_to_i64(raw_value)?;
            body.insert(
                if field_name == "layout_id" { "layoutId".to_string() } else { field_name.to_string() },
                json!(val)
            );
        }
        "transform" | "clip" => {
            let val = coerce_to_string(raw_value)?;
            body.insert(field_name.to_string(), json!(val));
        }
        "locked" | "always_loaded" | "alwaysLoaded" | "index_lock" | "indexLock" => {
            let val = coerce_to_bool(raw_value)?;
            let key = match field_name {
                "always_loaded" => "alwaysLoaded",
                "index_lock" => "indexLock",
                other => other,
            };
            body.insert(key.to_string(), json!(val));
        }

        // Text group fields
        "text" => {
            let val = coerce_to_string(raw_value)?;
            body.insert("text".to_string(), json!({ "text": val }));
        }
        "text_size" | "textSize" | "size" => {
            let val = coerce_to_i64(raw_value)?;
            body.insert("text".to_string(), json!({ "size": val }));
        }
        "color" | "font" | "css" => {
            let val = coerce_to_string(raw_value)?;
            body.insert("text".to_string(), json!({ field_name: val }));
        }

        // Image group fields
        "image_data_id" | "imageDataId" | "image_element_data_id" => {
            let val = coerce_to_u32(raw_value)?;
            body.insert("image".to_string(), json!({ "elementDataId": val }));
        }
        "image_url" | "imageUrl" | "url" => {
            let val = coerce_to_string(raw_value)?;
            body.insert("image".to_string(), json!({ "url": val }));
        }
        "image_width" | "imageWidth" => {
            let val = coerce_to_i64(raw_value)?;
            body.insert("image".to_string(), json!({ "width": val }));
        }
        "image_height" | "imageHeight" => {
            let val = coerce_to_i64(raw_value)?;
            body.insert("image".to_string(), json!({ "height": val }));
        }

        // Widget group fields
        "widget_data_id" | "widgetDataId" | "widget_element_data_id" => {
            let val = coerce_to_u32(raw_value)?;
            body.insert("widget".to_string(), json!({ "elementDataId": val }));
        }
        "widget_raw_data" | "widgetRawData" | "raw_data" => {
            let val = coerce_to_string(raw_value)?;
            body.insert("widget".to_string(), json!({ "rawData": val }));
        }
        "widget_width" | "widgetWidth" => {
            let val = coerce_to_i64(raw_value)?;
            body.insert("widget".to_string(), json!({ "width": val }));
        }
        "widget_height" | "widgetHeight" => {
            let val = coerce_to_i64(raw_value)?;
            body.insert("widget".to_string(), json!({ "height": val }));
        }

        // Media group fields
        "media_source" | "mediaSource" | "source" => {
            let val = coerce_to_string(raw_value)?;
            body.insert("media".to_string(), json!({ "source": val }));
        }
        "media_volume" | "mediaVolume" | "volume" => {
            let val = coerce_to_i64(raw_value)?;
            body.insert("media".to_string(), json!({ "volume": val }));
        }
        "media_playing" | "mediaPlaying" | "playing" => {
            let val = coerce_to_bool(raw_value)?;
            body.insert("media".to_string(), json!({ "playing": val }));
        }
        "media_timestamp" | "mediaTimestamp" | "timestamp" => {
            let val = coerce_to_i64(raw_value)?;
            body.insert("media".to_string(), json!({ "timestamp": val }));
        }
        "media_autoplay" | "mediaAutoplay" | "autoplay" => {
            let val = coerce_to_bool(raw_value)?;
            body.insert("media".to_string(), json!({ "autoplay": val }));
        }
        "media_loop" | "mediaLoop" | "loop" => {
            let val = coerce_to_bool(raw_value)?;
            body.insert("media".to_string(), json!({ "loop": val }));
        }
        "media_width" | "mediaWidth" => {
            let val = coerce_to_i64(raw_value)?;
            body.insert("media".to_string(), json!({ "width": val }));
        }
        "media_height" | "mediaHeight" => {
            let val = coerce_to_i64(raw_value)?;
            body.insert("media".to_string(), json!({ "height": val }));
        }

        _ => bail!("Unknown element update field: {field_name}"),
    }

    Ok(Value::Object(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coerce_to_string() {
        assert_eq!(coerce_to_string(&OscType::String("test".into())).unwrap(), "test");
        assert_eq!(coerce_to_string(&OscType::Int(123)).unwrap(), "123");
        assert_eq!(coerce_to_string(&OscType::Long(456)).unwrap(), "456");
        assert_eq!(coerce_to_string(&OscType::Float(1.23)).unwrap(), "1.23");
        assert_eq!(coerce_to_string(&OscType::Double(4.56)).unwrap(), "4.56");
        assert_eq!(coerce_to_string(&OscType::Bool(true)).unwrap(), "true");
    }

    #[test]
    fn test_coerce_to_u32() {
        assert_eq!(coerce_to_u32(&OscType::Int(123)).unwrap(), 123);
        assert_eq!(coerce_to_u32(&OscType::Long(456)).unwrap(), 456);
        assert_eq!(coerce_to_u32(&OscType::Float(7.89)).unwrap(), 7);
        assert_eq!(coerce_to_u32(&OscType::Double(9.87)).unwrap(), 9);
        assert_eq!(coerce_to_u32(&OscType::String("654".into())).unwrap(), 654);
    }

    #[test]
    fn test_coerce_to_i64() {
        assert_eq!(coerce_to_i64(&OscType::Int(123)).unwrap(), 123);
        assert_eq!(coerce_to_i64(&OscType::Long(456)).unwrap(), 456);
        assert_eq!(coerce_to_i64(&OscType::Float(7.89)).unwrap(), 7);
        assert_eq!(coerce_to_i64(&OscType::Double(9.87)).unwrap(), 9);
        assert_eq!(coerce_to_i64(&OscType::String("654".into())).unwrap(), 654);
    }

    #[test]
    fn test_coerce_to_bool() {
        assert_eq!(coerce_to_bool(&OscType::Bool(true)).unwrap(), true);
        assert_eq!(coerce_to_bool(&OscType::Int(1)).unwrap(), true);
        assert_eq!(coerce_to_bool(&OscType::Int(0)).unwrap(), false);
        assert_eq!(coerce_to_bool(&OscType::Float(0.0)).unwrap(), false);
        assert_eq!(coerce_to_bool(&OscType::Float(1.0)).unwrap(), true);
        assert_eq!(coerce_to_bool(&OscType::String("true".into())).unwrap(), true);
        assert_eq!(coerce_to_bool(&OscType::String("0".into())).unwrap(), false);
    }

    #[test]
    fn test_build_generic_update_body() {
        let body = build_generic_update_body("x", &OscType::Int(120)).unwrap();
        assert_eq!(body, json!({ "x": 120 }));

        let body = build_generic_update_body("text", &OscType::String("hello".into())).unwrap();
        assert_eq!(body, json!({ "text": { "text": "hello" } }));

        let body = build_generic_update_body("always_loaded", &OscType::Bool(true)).unwrap();
        assert_eq!(body, json!({ "alwaysLoaded": true }));

        let body = build_generic_update_body("media_playing", &OscType::Int(1)).unwrap();
        assert_eq!(body, json!({ "media": { "playing": true } }));
    }
}
