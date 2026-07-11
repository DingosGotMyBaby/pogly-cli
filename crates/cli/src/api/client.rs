use std::time::Duration;

use anyhow::{Context, Result};
use serde_json::Value;
use ureq::Agent;

#[derive(Debug)]
pub struct ApiError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (HTTP {})", self.message, self.status)
    }
}

impl std::error::Error for ApiError {}

pub struct ApiClient {
    agent: Agent,
    base: String,
    token: String,
}

impl ApiClient {
    pub fn new(host: &str, module: &str, token: &str) -> ApiClient {
        let config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(10)))
            .http_status_as_error(false)
            .user_agent(concat!("pogly-cli/", env!("CARGO_PKG_VERSION")))
            .build();
        ApiClient {
            agent: config.new_agent(),
            base: format!(
                "{}/v1/database/{}/route",
                host.trim_end_matches('/'),
                module
            ),
            token: token.to_string(),
        }
    }

    pub fn get(&self, path: &str) -> Result<Value> {
        self.call("GET", path, None)
    }

    pub fn post(&self, path: &str, body: Value) -> Result<Value> {
        self.call("POST", path, Some(body))
    }

    pub fn patch(&self, path: &str, body: Value) -> Result<Value> {
        self.call("PATCH", path, Some(body))
    }

    pub fn delete(&self, path: &str) -> Result<Value> {
        self.call("DELETE", path, None)
    }

    fn call(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
        let url = format!("{}/{}", self.base, path);
        let mut builder = ureq::http::Request::builder().method(method).uri(&url);
        if !self.token.is_empty() {
            builder = builder.header("Authorization", format!("Bearer {}", self.token));
        }
        let request = match body {
            Some(v) => builder
                .header("Content-Type", "application/json")
                .body(v.to_string())?,
            None => builder.body(String::new())?,
        };
        let mut response = self
            .agent
            .run(request)
            .with_context(|| format!("request to {url} failed"))?;
        let status = response.status().as_u16();
        let text = response
            .body_mut()
            .read_to_string()
            .context("failed to read response body")?;
        let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        if (200..300).contains(&status) {
            return Ok(json);
        }
        let message = json
            .get("error")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| crate::output::truncate(text.trim(), 200));
        Err(ApiError { status, message }.into())
    }
}

pub fn qs(pairs: &[(&str, Option<String>)]) -> String {
    let parts: Vec<String> = pairs
        .iter()
        .filter_map(|(k, v)| v.as_ref().map(|v| format!("{k}={}", encode(v))))
        .collect();
    if parts.is_empty() {
        String::new()
    } else {
        format!("?{}", parts.join("&"))
    }
}

fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
