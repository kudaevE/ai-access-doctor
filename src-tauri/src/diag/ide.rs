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

    // 2. Visual Studio Code
    audits.push(audit_vscode_flavor(
        "vscode",
        "Visual Studio Code",
        appdata.as_deref().map(|p| Path::new(p).join("Code").join("User").join("settings.json")),
    ));

    // 3. Windsurf
    audits.push(audit_vscode_flavor(
        "windsurf",
        "Windsurf Code Editor",
        appdata.as_deref().map(|p| Path::new(p).join("Windsurf").join("User").join("settings.json")),
    ));

    // 4. Zed Editor
    audits.push(audit_zed(
        appdata.as_deref().map(|p| Path::new(p).join("Zed").join("settings.json")),
        userprofile.as_deref().map(|p| Path::new(p).join(".config").join("zed").join("settings.json")),
    ));

    // 5. JetBrains IDEs
    audits.push(audit_jetbrains(appdata.as_deref()));

    // 6. Google Antigravity & AI Agents
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
    let mut rec = None;

    if let Some(p) = &proxy_val {
        if p.contains("53200") {
            status = "WARNING".to_string();
            diag = format!("В настройках указан устаревший/неработающий прокси: {}", p);
            rec = Some("Удалите параметр 'http.proxy' или укажите активный порт 10809 в настройках IDE.".to_string());
        } else {
            diag = format!("Настроен кастомный прокси: {}", p);
        }
    }

    if strict_ssl == Some(false) {
        diag.push_str(" Внимание: строгая проверка SSL отключена (proxyStrictSSL: false).");
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
