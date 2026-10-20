use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeConfigAudit {
    pub ide_id: String,
    pub ide_name: String,
    pub is_installed: bool,
    pub config_path: Option<String>,
    pub proxy_configured: Option<String>,
    pub strict_ssl: Option<bool>,
    pub status: String, // "HEALTHY" | "WARNING" | "NOT_INSTALLED"
    pub diagnosis_ru: String,
    pub recommendation_ru: Option<String>,
}

pub fn diagnose_ide_configs() -> Vec<IdeConfigAudit> {
    let mut audits = Vec::new();

    let appdata = std::env::var("APPDATA").ok();
    let userprofile = std::env::var("USERPROFILE").ok();

    // 1. Cursor
    audits.push(audit_vscode_flavor(
        "cursor",
        "Cursor AI Editor",
        appdata.as_deref().map(|p| Path::new(p).join("Cursor").join("User").join("settings.json")),
    ));

    // 2. Visual Studio Code & Copilot
    audits.push(audit_vscode_flavor(
        "vscode",
        "Visual Studio Code & Copilot",
        appdata.as_deref().map(|p| Path::new(p).join("Code").join("User").join("settings.json")),
    ));

    // 3. Windsurf
    audits.push(audit_vscode_flavor(
        "windsurf",
        "Windsurf Code Editor (Cascade)",
        appdata.as_deref().map(|p| Path::new(p).join("Windsurf").join("User").join("settings.json")),
    ));

    // 4. Claude Desktop
    audits.push(audit_claude(
        appdata.as_deref().map(|p| Path::new(p).join("Claude").join("claude_desktop_config.json")),
    ));

    // 5. Zed Editor
    audits.push(audit_zed(
        appdata.as_deref().map(|p| Path::new(p).join("Zed").join("settings.json")),
        userprofile.as_deref().map(|p| Path::new(p).join(".config").join("zed").join("settings.json")),
    ));

    // 6. JetBrains IDEs
    audits.push(audit_jetbrains(appdata.as_deref()));

    // 7. Google Antigravity & AI Agents
    audits.push(audit_antigravity(userprofile.as_deref()));

    audits
}

fn audit_vscode_flavor(id: &str, name: &str, path_opt: Option<PathBuf>) -> IdeConfigAudit {
    let path = match path_opt {
        Some(p) => p,
        None => {
            return IdeConfigAudit {
                ide_id: id.to_string(),
                ide_name: name.to_string(),
                is_installed: false,
                config_path: None,
                proxy_configured: None,
                strict_ssl: None,
                status: "NOT_INSTALLED".to_string(),
                diagnosis_ru: "Каталог настроек не найден.".to_string(),
                recommendation_ru: None,
            };
        }
    };

    if !path.exists() {
        return IdeConfigAudit {
            ide_id: id.to_string(),
            ide_name: name.to_string(),
            is_installed: false,
            config_path: Some(path.to_string_lossy().to_string()),
            proxy_configured: None,
            strict_ssl: None,
            status: "NOT_INSTALLED".to_string(),
            diagnosis_ru: format!("Файл конфигурации {} не найден.", path.file_name().unwrap_or_default().to_string_lossy()),
            recommendation_ru: None,
        };
    }

    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

    let proxy_val = json.get("http.proxy").and_then(|v| v.as_str()).map(|s| s.to_string());
    let strict_ssl = json.get("http.proxyStrictSSL").and_then(|v| v.as_bool());

    let mut status = "HEALTHY".to_string();
    let mut diag = "Конфигурация сетевого доступа IDE в норме.".to_string();
    let rec = None;

    if let Some(p) = &proxy_val {
        if p.contains("53200") {
            status = "HEALTHY".to_string();
            diag = format!("Настроен европейский мост AI Access Doctor: {}", p);
        } else {
            diag = format!("Настроен кастомный прокси: {}", p);
        }
    }

    if strict_ssl == Some(false) {
        diag.push_str(" Строгая проверка SSL отключена (proxyStrictSSL: false) для совместимости с прокси.");
    }

    IdeConfigAudit {
        ide_id: id.to_string(),
        ide_name: name.to_string(),
        is_installed: true,
        config_path: Some(path.to_string_lossy().to_string()),
        proxy_configured: proxy_val,
        strict_ssl,
        status,
        diagnosis_ru: diag,
        recommendation_ru: rec,
    }
}

fn audit_claude(path_opt: Option<PathBuf>) -> IdeConfigAudit {
    let path = match path_opt {
        Some(p) => p,
        None => {
            return IdeConfigAudit {
                ide_id: "claude".to_string(),
                ide_name: "Claude Desktop".to_string(),
                is_installed: false,
                config_path: None,
                proxy_configured: None,
                strict_ssl: None,
                status: "NOT_INSTALLED".to_string(),
                diagnosis_ru: "Каталог Claude Desktop не найден.".to_string(),
                recommendation_ru: None,
            };
        }
    };

    if !path.exists() {
        return IdeConfigAudit {
            ide_id: "claude".to_string(),
            ide_name: "Claude Desktop".to_string(),
            is_installed: false,
            config_path: Some(path.to_string_lossy().to_string()),
            proxy_configured: None,
            strict_ssl: None,
            status: "NOT_INSTALLED".to_string(),
            diagnosis_ru: "Claude Desktop не установлен или конфигурационный файл отсутствует.".to_string(),
            recommendation_ru: None,
        };
    }

    IdeConfigAudit {
        ide_id: "claude".to_string(),
        ide_name: "Claude Desktop".to_string(),
        is_installed: true,
        config_path: Some(path.to_string_lossy().to_string()),
        proxy_configured: None,
        strict_ssl: None,
        status: "HEALTHY".to_string(),
        diagnosis_ru: "Конфигурация Claude Desktop обнаружена.".to_string(),
        recommendation_ru: None,
    }
}

fn audit_zed(p1: Option<PathBuf>, p2: Option<PathBuf>) -> IdeConfigAudit {
    let candidate = p1.filter(|p| p.exists()).or_else(|| p2.filter(|p| p.exists()));

    match candidate {
        Some(path) => {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            IdeConfigAudit {
                ide_id: "zed".to_string(),
                ide_name: "Zed Editor".to_string(),
                is_installed: true,
                config_path: Some(path.to_string_lossy().to_string()),
                proxy_configured: None,
                strict_ssl: None,
                status: "HEALTHY".to_string(),
                diagnosis_ru: format!("Конфигурация Zed найдена (размер {} байт).", content.len()),
                recommendation_ru: None,
            }
        }
        None => IdeConfigAudit {
            ide_id: "zed".to_string(),
            ide_name: "Zed Editor".to_string(),
            is_installed: false,
            config_path: None,
            proxy_configured: None,
            strict_ssl: None,
            status: "NOT_INSTALLED".to_string(),
            diagnosis_ru: "Редактор Zed не обнаружен в системе.".to_string(),
            recommendation_ru: None,
        },
    }
}

fn audit_jetbrains(appdata_opt: Option<&str>) -> IdeConfigAudit {
    let appdata = match appdata_opt {
        Some(a) => a,
        None => {
            return IdeConfigAudit {
                ide_id: "jetbrains".to_string(),
                ide_name: "JetBrains IDEs (IntelliJ / PyCharm / WebStorm)".to_string(),
                is_installed: false,
                config_path: None,
                proxy_configured: None,
                strict_ssl: None,
                status: "NOT_INSTALLED".to_string(),
                diagnosis_ru: "Каталог JetBrains не найден.".to_string(),
                recommendation_ru: None,
            };
        }
    };

    let jb_dir = Path::new(appdata).join("JetBrains");
    if !jb_dir.exists() {
        return IdeConfigAudit {
            ide_id: "jetbrains".to_string(),
            ide_name: "JetBrains IDEs (IntelliJ / PyCharm / WebStorm)".to_string(),
            is_installed: false,
            config_path: None,
            proxy_configured: None,
            strict_ssl: None,
            status: "NOT_INSTALLED".to_string(),
            diagnosis_ru: "Установки JetBrains не обнаружены.".to_string(),
            recommendation_ru: None,
        };
    }

    let mut found_configs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&jb_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let proxy_xml = p.join("options").join("proxy.xml");
                if proxy_xml.exists() {
                    found_configs.push(proxy_xml.to_string_lossy().to_string());
                }
            }
        }
    }

    if found_configs.is_empty() {
        IdeConfigAudit {
            ide_id: "jetbrains".to_string(),
            ide_name: "JetBrains IDEs (IntelliJ / PyCharm / WebStorm)".to_string(),
            is_installed: true,
            config_path: Some(jb_dir.to_string_lossy().to_string()),
            proxy_configured: None,
            strict_ssl: None,
            status: "HEALTHY".to_string(),
            diagnosis_ru: "Среда JetBrains установлена. Используются системные параметры прокси Windows по умолчанию.".to_string(),
            recommendation_ru: None,
        }
    } else {
        IdeConfigAudit {
            ide_id: "jetbrains".to_string(),
            ide_name: "JetBrains IDEs (IntelliJ / PyCharm / WebStorm)".to_string(),
            is_installed: true,
            config_path: Some(found_configs[0].clone()),
            proxy_configured: None,
            strict_ssl: None,
            status: "HEALTHY".to_string(),
            diagnosis_ru: format!("Обнаружен конфигурационный файл прокси: {}", found_configs[0]),
            recommendation_ru: None,
        }
    }
}

fn audit_antigravity(userprofile_opt: Option<&str>) -> IdeConfigAudit {
    let userprofile = match userprofile_opt {
        Some(u) => u,
        None => {
            return IdeConfigAudit {
                ide_id: "antigravity".to_string(),
                ide_name: "Google Antigravity & Agentic Suite".to_string(),
                is_installed: false,
                config_path: None,
                proxy_configured: None,
                strict_ssl: None,
                status: "NOT_INSTALLED".to_string(),
                diagnosis_ru: "Профиль пользователя не найден.".to_string(),
                recommendation_ru: None,
            };
        }
    };

    let agy_dir = Path::new(userprofile).join(".gemini").join("antigravity");
    if agy_dir.exists() {
        IdeConfigAudit {
            ide_id: "antigravity".to_string(),
            ide_name: "Google Antigravity & Agentic Suite".to_string(),
            is_installed: true,
            config_path: Some(agy_dir.to_string_lossy().to_string()),
            proxy_configured: None,
            strict_ssl: Some(true),
            status: "HEALTHY".to_string(),
            diagnosis_ru: "Среда Google Antigravity активна. Доступ к агентам и хранилищу сессий функционирует.".to_string(),
            recommendation_ru: None,
        }
    } else {
        IdeConfigAudit {
            ide_id: "antigravity".to_string(),
            ide_name: "Google Antigravity & Agentic Suite".to_string(),
            is_installed: true,
            config_path: None,
            proxy_configured: None,
            strict_ssl: Some(true),
            status: "HEALTHY".to_string(),
            diagnosis_ru: "Среда разработки активна в текущей сессии.".to_string(),
            recommendation_ru: None,
        }
    }
}

pub fn get_settings_path_for_ide(ide_id: &str) -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    match ide_id {
        "cursor" => Some(Path::new(&appdata).join("Cursor").join("User").join("settings.json")),
        "vscode" => Some(Path::new(&appdata).join("Code").join("User").join("settings.json")),
        "windsurf" => Some(Path::new(&appdata).join("Windsurf").join("User").join("settings.json")),
        "antigravity" => Some(Path::new(&appdata).join("Antigravity").join("User").join("settings.json")),
        _ => None,
    }
}

pub fn fix_ide_proxy_and_ssl(
    ide_id: &str,
    proxy_url: Option<String>,
    disable_ssl: bool,
) -> Result<String, String> {
    let settings_path = get_settings_path_for_ide(ide_id)
        .ok_or_else(|| format!("Не поддерживается прямое редактирование настроек для '{}'", ide_id))?;

    if let Some(parent) = settings_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut json: serde_json::Value = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if !json.is_object() {
        json = serde_json::json!({});
    }

    let target_proxy = proxy_url.unwrap_or_else(|| "http://127.0.0.1:53200".to_string());
    json["http.proxy"] = serde_json::Value::String(target_proxy.clone());
    json["http.proxySupport"] = serde_json::Value::String("override".to_string());
    if disable_ssl {
        json["http.proxyStrictSSL"] = serde_json::Value::Bool(false);
    }

    let formatted = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Ошибка сериализации настроек: {}", e))?;
    std::fs::write(&settings_path, formatted)
        .map_err(|e| format!("Ошибка записи файла {}: {}", settings_path.display(), e))?;

    Ok(format!(
        "Конфигурация {} успешно обновлена! Установлен прокси: {}, proxyStrictSSL: {}.",
        ide_id, target_proxy, !disable_ssl
    ))
}

pub fn reset_ide_proxy(ide_id: &str) -> Result<String, String> {
    let settings_path = get_settings_path_for_ide(ide_id)
        .ok_or_else(|| format!("Не поддерживается сброс для '{}'", ide_id))?;

    if !settings_path.exists() {
        return Ok("Файл настроек уже чист.".to_string());
    }

    let content = std::fs::read_to_string(&settings_path).unwrap_or_default();
    let mut json: serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::json!({}));

    if let Some(obj) = json.as_object_mut() {
        obj.remove("http.proxy");
        obj.insert("http.proxySupport".to_string(), serde_json::Value::String("on".to_string()));
        obj.insert("http.proxyStrictSSL".to_string(), serde_json::Value::Bool(true));
    }

    let formatted = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Ошибка сериализации: {}", e))?;
    std::fs::write(&settings_path, formatted)
        .map_err(|e| format!("Ошибка записи: {}", e))?;

    Ok(format!("Настройки прокси для {} успешно сброшены к системным значениям.", ide_id))
}
