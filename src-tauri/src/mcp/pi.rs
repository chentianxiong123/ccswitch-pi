//! Pi MCP 同步和导入模块
//!
//! Pi 本身不支持原生 MCP。本模块将 MCP 服务器配置写入
//! `~/.pi/agent/mcp-servers.json`，由 pi extension 读取和连接。

use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;

use crate::error::AppError;
use crate::pi_config::get_pi_dir;

use super::validation::validate_server_spec;

// ============================================================================
// Path
// ============================================================================

fn get_mcp_servers_path() -> std::path::PathBuf {
    get_pi_dir().join("mcp-servers.json")
}

// ============================================================================
// Format Conversion: CC Switch → Pi MCP
// ============================================================================

/// Convert CC Switch unified format to Pi MCP format
fn convert_to_pi_mcp_format(spec: &Value) -> Result<Value, AppError> {
    let obj = spec
        .as_object()
        .ok_or_else(|| AppError::McpValidation("MCP spec must be a JSON object".into()))?;

    let typ = obj.get("type").and_then(|v| v.as_str()).unwrap_or("stdio");

    let mut result = serde_json::Map::new();

    match typ {
        "stdio" => {
            result.insert("type".into(), json!("stdio"));
            if let Some(cmd) = obj.get("command") {
                result.insert("command".into(), cmd.clone());
            }
            if let Some(args) = obj.get("args") {
                result.insert("args".into(), args.clone());
            }
            if let Some(env) = obj.get("env") {
                if env.is_object() && !env.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                    result.insert("env".into(), env.clone());
                }
            }
        }
        "sse" | "http" => {
            result.insert("type".into(), json!("sse"));
            if let Some(url) = obj.get("url") {
                result.insert("url".into(), url.clone());
            }
            if let Some(headers) = obj.get("headers") {
                if headers.is_object() && !headers.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                    result.insert("headers".into(), headers.clone());
                }
            }
        }
        _ => {
            // Unknown type, passthrough
            result.insert("type".into(), json!(typ));
            for (k, v) in obj {
                result.insert(k.clone(), v.clone());
            }
        }
    }

    Ok(Value::Object(result))
}

// ============================================================================
// Sync Functions
// ============================================================================

/// Write all enabled MCP servers to pi's mcp-servers.json
pub fn sync_enabled_to_pi(enabled: &HashMap<String, Value>) -> Result<(), AppError> {
    let path = get_mcp_servers_path();
    if enabled.is_empty() {
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                AppError::Config(format!("Failed to remove pi MCP config: {e}"))
            })?;
        }
        return Ok(());
    }

    let mut output = serde_json::Map::new();
    for (id, entry) in enabled {
        match convert_to_pi_mcp_format(entry) {
            Ok(spec) => {
                output.insert(id.clone(), spec);
            }
            Err(err) => {
                log::warn!("Skipping invalid MCP entry '{id}': {err}");
            }
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::Config(format!("Failed to create pi MCP dir: {e}"))
        })?;
    }

    let content = serde_json::to_string_pretty(&Value::Object(output)).map_err(|e| {
        AppError::Config(format!("Failed to serialize pi MCP config: {e}"))
    })?;

    fs::write(&path, content).map_err(|e| {
        AppError::Config(format!("Failed to write pi MCP config: {e}"))
    })?;

    log::info!("Pi MCP servers written to {}", path.display());
    Ok(())
}

/// Sync a single MCP server to pi
pub fn sync_single_server_to_pi(
    _config: &crate::app_config::MultiAppConfig,
    server_id: &str,
    server_spec: &Value,
) -> Result<(), AppError> {
    let path = get_mcp_servers_path();

    let mut servers: HashMap<String, Value> = if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| {
            AppError::Config(format!("Failed to read pi MCP config: {e}"))
        })?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };

    if let Ok(spec) = convert_to_pi_mcp_format(server_spec) {
        servers.insert(server_id.to_string(), spec);
    }

    sync_enabled_to_pi(&servers)
}

/// Remove a server from pi's MCP config
pub fn remove_server_from_pi(server_id: &str) -> Result<(), AppError> {
    let path = get_mcp_servers_path();
    if !path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&path).map_err(|e| {
        AppError::Config(format!("Failed to read pi MCP config: {e}"))
    })?;

    let mut servers: HashMap<String, Value> = serde_json::from_str(&content).unwrap_or_default();

    if servers.remove(server_id).is_some() {
        sync_enabled_to_pi(&servers)?;
    }

    Ok(())
}