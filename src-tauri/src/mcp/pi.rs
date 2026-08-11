//! Pi MCP 同步和导入模块
//!
//! Pi 通过 `pi-mcp-adapter` 包支持 MCP。
//! 本模块将 MCP 服务器配置写入 `~/.pi/agent/mcp.json`（标准 MCP 格式），
//! pi-mcp-adapter 自动读取并加载。

use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;

use crate::error::AppError;
use crate::pi_config::get_pi_dir;

use super::validation::validate_server_spec;

// ============================================================================
// Path
// ============================================================================

fn get_mcp_config_path() -> std::path::PathBuf {
    get_pi_dir().join("mcp.json")
}

// ============================================================================
// Format Conversion: CC Switch → Standard MCP (Claude-compatible)
// ============================================================================

/// Convert CC Switch unified format to standard MCP format
fn convert_to_standard_mcp_format(spec: &Value) -> Result<Value, AppError> {
    let obj = spec
        .as_object()
        .ok_or_else(|| AppError::McpValidation("MCP spec must be a JSON object".into()))?;

    let typ = obj.get("type").and_then(|v| v.as_str()).unwrap_or("stdio");

    let mut result = serde_json::Map::new();

    match typ {
        "stdio" => {
            if let Some(cmd) = obj.get("command").and_then(|v| v.as_str()) {
                result.insert("command".into(), json!(cmd));
            }
            if let Some(args) = obj.get("args").and_then(|v| v.as_array()) {
                result.insert("args".into(), json!(args));
            }
            if let Some(env) = obj.get("env") {
                if env.is_object() && !env.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                    result.insert("env".into(), env.clone());
                }
            }
        }
        "sse" | "http" => {
            if let Some(url) = obj.get("url").and_then(|v| v.as_str()) {
                result.insert("url".into(), json!(url));
            }
            if let Some(headers) = obj.get("headers") {
                if headers.is_object() && !headers.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                    result.insert("headers".into(), headers.clone());
                }
            }
        }
        _ => {
            for (k, v) in obj {
                result.insert(k.clone(), v.clone());
            }
        }
    }

    Ok(Value::Object(result))
}

// ============================================================================
// Read / Write
// ============================================================================

fn read_existing_config(path: &std::path::Path) -> HashMap<String, Value> {
    if !path.exists() {
        return HashMap::new();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|content| {
            serde_json::from_str::<Value>(&content)
                .ok()
                .and_then(|v| {
                    v.get("mcpServers")
                        .and_then(|s| s.as_object())
                        .map(|o| {
                            o.iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect::<HashMap<_, _>>()
                        })
                })
        })
        .unwrap_or_default()
}

fn write_config(path: &std::path::Path, servers: &HashMap<String, Value>) -> Result<(), AppError> {
    let mut output = serde_json::Map::new();
    let mut mcp_servers = serde_json::Map::new();
    for (id, spec) in servers {
        mcp_servers.insert(id.clone(), spec.clone());
    }
    output.insert("mcpServers".into(), Value::Object(mcp_servers));

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::Config(format!("Failed to create dir: {e}"))
        })?;
    }

    let content = serde_json::to_string_pretty(&Value::Object(output)).map_err(|e| {
        AppError::Config(format!("Failed to serialize: {e}"))
    })?;

    fs::write(path, content).map_err(|e| {
        AppError::Config(format!("Failed to write: {e}"))
    })?;

    log::info!("Pi MCP config written to {}", path.display());
    Ok(())
}

// ============================================================================
// Sync Functions
// ============================================================================

/// Sync a single MCP server to pi
pub fn sync_single_server_to_pi(
    _config: &crate::app_config::MultiAppConfig,
    server_id: &str,
    server_spec: &Value,
) -> Result<(), AppError> {
    let path = get_mcp_config_path();
    let mut servers = read_existing_config(&path);

    if let Ok(spec) = convert_to_standard_mcp_format(server_spec) {
        servers.insert(server_id.to_string(), spec);
    }

    write_config(&path, &servers)
}

/// Remove a server from pi's MCP config
pub fn remove_server_from_pi(server_id: &str) -> Result<(), AppError> {
    let path = get_mcp_config_path();
    if !path.exists() {
        return Ok(());
    }

    let mut servers = read_existing_config(&path);
    if servers.remove(server_id).is_some() {
        if servers.is_empty() {
            if path.exists() {
                fs::remove_file(&path).ok();
            }
        } else {
            write_config(&path, &servers)?;
        }
    }

    Ok(())
}

/// Sync all enabled servers to pi (writes full mcpServers map)
pub fn sync_enabled_to_pi(enabled: &HashMap<String, Value>) -> Result<(), AppError> {
    let path = get_mcp_config_path();
    if enabled.is_empty() {
        if path.exists() {
            fs::remove_file(&path).ok();
        }
        return Ok(());
    }

    let mut servers = HashMap::new();
    for (id, entry) in enabled {
        match convert_to_standard_mcp_format(entry) {
            Ok(spec) => {
                servers.insert(id.clone(), spec);
            }
            Err(err) => {
                log::warn!("Skipping invalid MCP entry '{id}': {err}");
            }
        }
    }

    write_config(&path, &servers)
}