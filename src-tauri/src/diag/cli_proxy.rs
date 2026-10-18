use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliProxyStatus {
    pub git_proxy: Option<String>,
    pub npm_proxy: Option<String>,
    pub env_http_proxy: Option<String>,
    pub env_https_proxy: Option<String>,
    pub is_any_configured: bool,
    pub effective_summary_ru: String,
}

const CREATE_NO_WINDOW: u32 = 0x08000000;
use std::os::windows::process::CommandExt;

pub fn get_cli_proxy_status() -> CliProxyStatus {
    // 1. Git proxy
    let git_out = Command::new("git")
        .args(["config", "--global", "--get", "http.proxy"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    let git_proxy = git_out
        .ok()
        .and_then(|o| if o.status.success() {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        } else { None });

    // 2. npm proxy
    let npm_out = Command::new("cmd")
        .args(["/C", "npm config get proxy"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    let npm_proxy = npm_out
        .ok()
        .and_then(|o| if o.status.success() {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() || s == "null" || s == "undefined" { None } else { Some(s) }
        } else { None });

    // 3. User Environment Variables
    let env_http = std::env::var("HTTP_PROXY").ok().or_else(|| std::env::var("http_proxy").ok());
    let env_https = std::env::var("HTTPS_PROXY").ok().or_else(|| std::env::var("https_proxy").ok());

    let is_any = git_proxy.is_some() || npm_proxy.is_some() || env_http.is_some() || env_https.is_some();

    let effective_summary_ru = if is_any {
        format!(
            "Прокси настроен: Git ({}), npm ({}), Env ({})",
            git_proxy.as_deref().unwrap_or("нет"),
            npm_proxy.as_deref().unwrap_or("нет"),
            env_https.as_deref().unwrap_or("нет")
        )
    } else {
        "Терминальные утилиты (Git, npm, pip) используют прямое соединение (прокси не задан).".to_string()
    };

    CliProxyStatus {
        git_proxy,
        npm_proxy,
        env_http_proxy: env_http,
        env_https_proxy: env_https,
        is_any_configured: is_any,
        effective_summary_ru,
    }
}

pub fn apply_cli_proxy(proxy_url: &str) -> Result<String, String> {
    let clean_url = proxy_url.trim();
    if clean_url.is_empty() {
        return Err("Адрес прокси не может быть пустым.".to_string());
    }

    // Git
    let _ = Command::new("git")
        .args(["config", "--global", "http.proxy", clean_url])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    let _ = Command::new("git")
        .args(["config", "--global", "https.proxy", clean_url])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    // npm
    let _ = Command::new("cmd")
        .args(["/C", &format!("npm config set proxy {}", clean_url)])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    let _ = Command::new("cmd")
        .args(["/C", &format!("npm config set https-proxy {}", clean_url)])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    // PowerShell / Windows User Environment
    let ps_script = format!(
        "[System.Environment]::SetEnvironmentVariable('HTTP_PROXY', '{}', 'User'); \
         [System.Environment]::SetEnvironmentVariable('HTTPS_PROXY', '{}', 'User'); \
         [System.Environment]::SetEnvironmentVariable('ALL_PROXY', '{}', 'User');",
        clean_url, clean_url, clean_url
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    Ok(format!(
        "Прокси '{}' успешно привязан к Git, npm и системным переменным консоли!",
        clean_url
    ))
}

pub fn reset_cli_proxy() -> Result<String, String> {
    // Git
    let _ = Command::new("git")
        .args(["config", "--global", "--unset", "http.proxy"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    let _ = Command::new("git")
        .args(["config", "--global", "--unset", "https.proxy"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    // npm
    let _ = Command::new("cmd")
        .args(["/C", "npm config delete proxy"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    let _ = Command::new("cmd")
        .args(["/C", "npm config delete https-proxy"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    // Environment
    let ps_script = "\
        [System.Environment]::SetEnvironmentVariable('HTTP_PROXY', $null, 'User'); \
        [System.Environment]::SetEnvironmentVariable('HTTPS_PROXY', $null, 'User'); \
        [System.Environment]::SetEnvironmentVariable('ALL_PROXY', $null, 'User');";
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_script])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    Ok("Настройки прокси для Git, npm и терминальных переменных сброшены (прямое подключение).".to_string())
}
