use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerAudit {
    pub name: String,
    pub transport_type: String, // "stdio" | "sse"
    pub command_or_url: String,
    pub is_executable_found: bool,
    pub status: String, // "HEALTHY" | "EXECUTABLE_MISSING" | "WARNING"
    pub details_ru: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpFileAudit {
    pub client_name: String,
    pub file_path: String,
    pub exists: bool,
    pub is_valid_json: bool,
    pub servers_count: usize,
    pub servers: Vec<McpServerAudit>,
    pub recommendation_ru: Option<String>,
}

pub fn diagnose_mcp_configs() -> Vec<McpFileAudit> {
    let mut audits = Vec::new();
    let appdata = std::env::var("APPDATA").ok();
    let userprofile = std::env::var("USERPROFILE").ok();

    // 1. Claude Desktop MCP Config
    if let Some(app) = &appdata {
        let claude_path = Path::new(app).join("Claude").join("claude_desktop_config.json");
        audits.push(audit_mcp_file("Claude Desktop", claude_path));
    }

    // 2. Cursor MCP Config
    if let Some(user) = &userprofile {
        let cursor_path = Path::new(user).join(".cursor").join("mcp.json");
        audits.push(audit_mcp_file("Cursor AI", cursor_path));
    }

    // 3. Antigravity MCP Config
    if let Some(user) = &userprofile {
        let agy_path = Path::new(user).join(".gemini").join("antigravity").join("mcp_servers.json");
        audits.push(audit_mcp_file("Google Antigravity MCP", agy_path));
    }

    audits
}

fn audit_mcp_file(client_name: &str, path: PathBuf) -> McpFileAudit {
    let exists = path.exists();
    let path_str = path.to_string_lossy().to_string();

    if !exists {
        return McpFileAudit {
            client_name: client_name.to_string(),
            file_path: path_str,
            exists: false,
            is_valid_json: false,
            servers_count: 0,
            servers: Vec::new(),
            recommendation_ru: None,
        };
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            return McpFileAudit {
                client_name: client_name.to_string(),
                file_path: path_str,
                exists: true,
                is_valid_json: false,
                servers_count: 0,
                servers: Vec::new(),
                recommendation_ru: Some(format!("Не удалось прочитать файл конфигурации: {}", e)),
            };
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(e) => {
            return McpFileAudit {
                client_name: client_name.to_string(),
                file_path: path_str,
                exists: true,
                is_valid_json: false,
                servers_count: 0,
                servers: Vec::new(),
                recommendation_ru: Some(format!("Ошибка синтаксиса JSON: {}", e)),
            };
        }
    };

    let mut server_audits = Vec::new();
    if let Some(mcp_servers) = json.get("mcpServers").and_then(|v| v.as_object()) {
        for (name, cfg) in mcp_servers {
            let command = cfg.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let url = cfg.get("url").and_then(|v| v.as_str()).unwrap_or("");

            if !url.is_empty() {
                // SSE transport
                server_audits.push(McpServerAudit {
                    name: name.clone(),
                    transport_type: "sse".to_string(),
                    command_or_url: url.to_string(),
                    is_executable_found: true,
                    status: "HEALTHY".to_string(),
                    details_ru: format!("Сетевой транспорт SSE: {}", url),
                });
            } else {
                // stdio transport
                let exe_exists = check_executable_exists(command);
                let status = if exe_exists { "HEALTHY".to_string() } else { "EXECUTABLE_MISSING".to_string() };
                let details = if exe_exists {
                    format!("Исполняемый файл '{}' обнаружен в PATH.", command)
                } else {
                    format!("Исполняемый файл '{}' не найден в путях PATH Windows.", command)
                };

                server_audits.push(McpServerAudit {
                    name: name.clone(),
                    transport_type: "stdio".to_string(),
                    command_or_url: command.to_string(),
                    is_executable_found: exe_exists,
                    status,
                    details_ru: details,
                });
            }
        }
    }

    let servers_count = server_audits.len();
    McpFileAudit {
        client_name: client_name.to_string(),
        file_path: path_str,
        exists: true,
        is_valid_json: true,
        servers_count,
        servers: server_audits,
        recommendation_ru: None,
    }
}

fn check_executable_exists(command: &str) -> bool {
    if command.is_empty() {
        return false;
    }
    // If absolute path
    if Path::new(command).exists() {
        return true;
    }

    // Common global runtimes
    if command == "node" || command == "npx" || command == "python" || command == "python3" || command == "cmd" || command == "powershell" || command == "uvx" {
        return true; // standard runtimes on developer Windows machine
    }

    // Check system PATH
    if let Ok(paths) = std::env::var("PATH") {
        for p in paths.split(';') {
            let direct = Path::new(p).join(command);
            let with_exe = Path::new(p).join(format!("{}.exe", command));
            let with_cmd = Path::new(p).join(format!("{}.cmd", command));
            if direct.exists() || with_exe.exists() || with_cmd.exists() {
                return true;
            }
        }
    }

    false
}
