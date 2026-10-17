use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinInetProxy {
    pub enabled: bool,
    pub server: Option<String>,
    pub pac_url: Option<String>,
    pub override_exceptions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinHttpProxy {
    pub is_direct: bool,
    pub proxy_server: Option<String>,
    pub bypass_list: Option<String>,
    pub raw_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyEnvVars {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub all_proxy: Option<String>,
    pub no_proxy: Option<String>,
    pub has_any: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyEndpointCheck {
    pub endpoint: String,
    pub is_listening: bool,
    pub tcp_rtt_ms: Option<u64>,
    pub can_forward_https: bool,
    pub http_status: Option<u16>,
    pub response_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyDiagnostics {
    pub wininet: WinInetProxy,
    pub winhttp: WinHttpProxy,
    pub env_vars: ProxyEnvVars,
    pub active_proxies: Vec<ProxyEndpointCheck>,
    pub has_conflict: bool,
    pub conflict_description: Option<String>,
    pub recommendation: Option<String>,
}

/// Query Windows User Proxy (WinINet) from HKCU registry
pub fn get_wininet_proxy() -> WinInetProxy {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(settings) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings") {
            let enabled: u32 = settings.get_value("ProxyEnable").unwrap_or(0);
            let server: Option<String> = settings.get_value("ProxyServer").ok();
            let pac_url: Option<String> = settings.get_value("AutoConfigURL").ok();
            let override_exceptions: Option<String> = settings.get_value("ProxyOverride").ok();

            return WinInetProxy {
                enabled: enabled == 1,
                server,
                pac_url,
                override_exceptions,
            };
        }
    }

    WinInetProxy {
        enabled: false,
        server: None,
        pac_url: None,
        override_exceptions: None,
    }
}

/// Query WinHTTP proxy via `netsh winhttp show proxy`
pub fn get_winhttp_proxy() -> WinHttpProxy {
    let output = super::silent_command("netsh")
        .args(["winhttp", "show", "proxy"])
        .output();

    if let Ok(out) = output {
        let stdout_str = String::from_utf8_lossy(&out.stdout).to_string();
        let is_direct = stdout_str.to_lowercase().contains("direct")
            || stdout_str.to_lowercase().contains("прямой доступ");

        let mut proxy_server = None;
        let mut bypass_list = None;

        for line in stdout_str.lines() {
            let l = line.trim();
            if l.to_lowercase().contains("proxy server(s)") || l.to_lowercase().contains("прокси-сервер(ы)") {
                if let Some(pos) = l.find(':') {
                    proxy_server = Some(l[pos + 1..].trim().to_string());
                }
            } else if l.to_lowercase().contains("bypass list") || l.to_lowercase().contains("список обхода") {
                if let Some(pos) = l.find(':') {
                    bypass_list = Some(l[pos + 1..].trim().to_string());
                }
            }
        }

        WinHttpProxy {
            is_direct,
            proxy_server,
            bypass_list,
            raw_output: stdout_str,
        }
    } else {
        WinHttpProxy {
            is_direct: true,
            proxy_server: None,
            bypass_list: None,
            raw_output: "Unable to run netsh winhttp".to_string(),
        }
    }
}

/// Inspect environment proxy variables (uppercase and lowercase)
pub fn get_env_proxy_vars() -> ProxyEnvVars {
    let vars: HashMap<String, String> = std::env::vars().collect();

    let get_var = |key: &str| -> Option<String> {
        vars.get(key)
            .or_else(|| vars.get(&key.to_lowercase()))
            .cloned()
            .filter(|s| !s.trim().is_empty())
    };

    let http_proxy = get_var("HTTP_PROXY");
    let https_proxy = get_var("HTTPS_PROXY");
    let all_proxy = get_var("ALL_PROXY");
    let no_proxy = get_var("NO_PROXY");

    let has_any = http_proxy.is_some() || https_proxy.is_some() || all_proxy.is_some();

    ProxyEnvVars {
        http_proxy,
        https_proxy,
        all_proxy,
        no_proxy,
        has_any,
    }
}

/// Parse host and port from proxy string (e.g. "http://127.0.0.1:10809", "127.0.0.1:10809")
fn parse_proxy_host_port(proxy_str: &str) -> Option<(String, u16)> {
    let cleaned = proxy_str
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_start_matches("socks5://")
        .trim_start_matches("socks5h://")
        .trim_end_matches('/');

    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() == 2 {
        let host = parts[0].to_string();
        if let Ok(port) = parts[1].parse::<u16>() {
            return Some((host, port));
        }
    }
    None
}

/// Probe a specific proxy endpoint: checks TCP connectivity and HTTP CONNECT / GET tunnel
pub async fn probe_proxy_endpoint(proxy_url: &str) -> ProxyEndpointCheck {
    let (host, port) = match parse_proxy_host_port(proxy_url) {
        Some(hp) => hp,
        None => {
            return ProxyEndpointCheck {
                endpoint: proxy_url.to_string(),
                is_listening: false,
                tcp_rtt_ms: None,
                can_forward_https: false,
                http_status: None,
                response_error: Some("Unable to parse proxy host and port".to_string()),
            };
        }
    };

    let start = Instant::now();
    let addr_str = format!("{}:{}", host, port);

    // 1. TCP socket reachability test
    let tcp_conn = tokio::time::timeout(Duration::from_millis(1500), TcpStream::connect(&addr_str)).await;

    let (is_listening, tcp_rtt_ms) = match tcp_conn {
        Ok(Ok(_stream)) => (true, Some(start.elapsed().as_millis() as u64)),
        _ => (false, None),
    };

    if !is_listening {
        return ProxyEndpointCheck {
            endpoint: proxy_url.to_string(),
            is_listening: false,
            tcp_rtt_ms: None,
            can_forward_https: false,
            http_status: None,
            response_error: Some("Proxy port is closed or connection refused (Daemon dead)".to_string()),
        };
    }

    // 2. HTTP CONNECT forwarding test using reqwest through this specific proxy
    let proxy_spec = if proxy_url.starts_with("http://") || proxy_url.starts_with("socks5://") {
        proxy_url.to_string()
    } else {
        format!("http://{}", proxy_url)
    };

    let client_builder = reqwest::Client::builder()
        .timeout(Duration::from_millis(3000));

    let client = match reqwest::Proxy::all(&proxy_spec) {
        Ok(p) => client_builder.proxy(p).build(),
        Err(e) => {
            return ProxyEndpointCheck {
                endpoint: proxy_url.to_string(),
                is_listening: true,
                tcp_rtt_ms,
                can_forward_https: false,
                http_status: None,
                response_error: Some(format!("Invalid proxy syntax: {}", e)),
            };
        }
    };

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return ProxyEndpointCheck {
                endpoint: proxy_url.to_string(),
                is_listening: true,
                tcp_rtt_ms,
                can_forward_https: false,
                http_status: None,
                response_error: Some(format!("Client build error: {}", e)),
            };
        }
    };

    // Test request to Cloudflare trace or npmjs
    let test_res = client.get("https://cloudflare.com/cdn-cgi/trace").send().await;

    match test_res {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let can_forward = resp.status().is_success();
            let err = if !can_forward {
                Some(format!("Proxy returned HTTP status {}", status))
            } else {
                None
            };

            ProxyEndpointCheck {
                endpoint: proxy_url.to_string(),
                is_listening: true,
                tcp_rtt_ms,
                can_forward_https: can_forward,
                http_status: Some(status),
                response_error: err,
            }
        }
        Err(e) => ProxyEndpointCheck {
            endpoint: proxy_url.to_string(),
            is_listening: true,
            tcp_rtt_ms,
            can_forward_https: false,
            http_status: e.status().map(|s| s.as_u16()),
            response_error: Some(format!("Forwarding failed: {}", e)),
        },
    }
}

/// Run full proxy audit and detect conflicting or broken proxy configurations
pub async fn diagnose_proxies() -> ProxyDiagnostics {
    let wininet = get_wininet_proxy();
    let winhttp = get_winhttp_proxy();
    let env_vars = get_env_proxy_vars();

    let mut detected_urls = Vec::new();

    if let Some(s) = &wininet.server {
        if wininet.enabled {
            detected_urls.push(s.clone());
        }
    }
    if let Some(s) = &env_vars.http_proxy {
        detected_urls.push(s.clone());
    }
    if let Some(s) = &env_vars.https_proxy {
        detected_urls.push(s.clone());
    }
    if let Some(s) = &env_vars.all_proxy {
        detected_urls.push(s.clone());
    }

    detected_urls.dedup();

    let mut active_proxies = Vec::new();
    for url in detected_urls {
        let check = probe_proxy_endpoint(&url).await;
        active_proxies.push(check);
    }

    // Conflict analysis
    let mut has_conflict = false;
    let mut conflict_desc = None;
    let mut recommendation = None;

    // Check 1: Env proxy points to a dead port or 502 while WinINet points to a working port
    let dead_env_proxy = active_proxies.iter().find(|p| {
        !p.can_forward_https
            && (env_vars.http_proxy.as_deref() == Some(&p.endpoint)
                || env_vars.https_proxy.as_deref() == Some(&p.endpoint))
    });

    let working_wininet_proxy = active_proxies.iter().find(|p| {
        p.can_forward_https && wininet.server.as_deref() == Some(&p.endpoint)
    });

    if let (Some(dead), Some(working)) = (dead_env_proxy, working_wininet_proxy) {
        has_conflict = true;
        conflict_desc = Some(format!(
            "Конфликт прокси: Переменная окружения HTTPS_PROXY указывает на неработающий прокси ({}), тогда как системный прокси Windows ({}) функционирует корректно.",
            dead.endpoint, working.endpoint
        ));
        recommendation = Some(format!(
            "Обновите переменные окружения HTTP_PROXY и HTTPS_PROXY на working адрес {} или удалите устаревшие переменные через Панель управления.",
            working.endpoint
        ));
    } else if let Some(dead) = dead_env_proxy {
        has_conflict = true;
        conflict_desc = Some(format!(
            "Переменная окружения HTTPS_PROXY указывает на адрес {}, порт которого не отвечает или возвращает ошибку.",
            dead.endpoint
        ));
        recommendation = Some("Запустите локальный прокси-клиент (v2ray/sing-box/Xray) или сбросьте HTTP_PROXY / HTTPS_PROXY.".to_string());
    } else if wininet.enabled && !active_proxies.iter().any(|p| p.can_forward_https) {
        has_conflict = true;
        conflict_desc = Some("В настройках Windows включен системный прокси, но прокси-сервер не отвечает на запросы.".to_string());
        recommendation = Some("Отключите системный прокси в Параметрах Windows (Сеть и интернет -> Прокси-сервер) или проверьте статус VPN/прокси клиента.".to_string());
    }

    ProxyDiagnostics {
        wininet,
        winhttp,
        env_vars,
        active_proxies,
        has_conflict,
        conflict_description: conflict_desc,
        recommendation,
    }
}
