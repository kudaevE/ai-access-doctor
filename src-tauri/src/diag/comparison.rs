use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelProbeResult {
    pub channel_id: String,
    pub channel_name: String,
    pub channel_type: String, // "Direct" | "SystemProxy" | "EnvProxy" | "DoH" | "VPN"
    pub is_available: bool,
    pub latency_ms: u64,
    pub http_status: Option<u16>,
    pub status_text: String, // "SUCCESS" | "FAILED" | "TIMEOUT" | "SKIPPED"
    pub details_ru: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMatrixReport {
    pub target_url: String,
    pub target_name: String,
    pub timestamp: String,
    pub results: Vec<ChannelProbeResult>,
    pub comparative_verdict_ru: String,
}

pub async fn run_connection_matrix(target_url_opt: Option<&str>) -> ConnectionMatrixReport {
    let target = target_url_opt.unwrap_or("https://accounts.google.com");
    let target_name = if target.contains("google") {
        "Google Antigravity & Services"
    } else if target.contains("openai") {
        "OpenAI / ChatGPT"
    } else if target.contains("anthropic") {
        "Anthropic Claude"
    } else {
        "Target AI Service"
    };

    let mut results = Vec::new();

    // 1. Channel: Direct (Bypassing any proxy)
    let direct_res = probe_direct(target).await;
    results.push(direct_res);

    // 2. Channel: Windows WinINet System Proxy
    let wininet_res = probe_system_proxy(target).await;
    results.push(wininet_res);

    // 3. Channel: Environment Variable Proxy
    let env_res = probe_env_proxy(target).await;
    results.push(env_res);

    // 4. Channel: Cloudflare DoH (DNS-over-HTTPS)
    let doh_res = probe_doh("accounts.google.com").await;
    results.push(doh_res);

    // 5. Channel: VPN / TUN detection
    let vpn_res = probe_vpn_route().await;
    results.push(vpn_res);

    // Synthesize comparative verdict
    let direct_ok = results[0].is_available;
    let proxy_ok = results[1].is_available;
    let env_ok = results[2].is_available;

    let verdict = if direct_ok && proxy_ok {
        "Все каналы связи (и прямой доступ, и системный прокси) работают без ограничений.".to_string()
    } else if !direct_ok && proxy_ok {
        "Прямой доступ блокируется провайдером, но системный прокси успешно обходит ограничения и обеспечивает стабильное соединение.".to_string()
    } else if proxy_ok && !env_ok && results[2].status_text != "SKIPPED" {
        "Обнаружена дивергенция каналов: системный прокси Windows исправен, однако переменная окружения HTTP_PROXY в терминале сломана.".to_string()
    } else if !direct_ok && !proxy_ok {
        "Все доступные каналы связи заблокированы или недоступны. Требуется настройка VPN или рабочего прокси.".to_string()
    } else {
        "Сравнительный анализ каналов завершен.".to_string()
    };

    ConnectionMatrixReport {
        target_url: target.to_string(),
        target_name: target_name.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        results,
        comparative_verdict_ru: verdict,
    }
}

async fn probe_direct(url: &str) -> ChannelProbeResult {
    let start = Instant::now();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(3500))
        .no_proxy()
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return ChannelProbeResult {
                channel_id: "direct".to_string(),
                channel_name: "Прямое соединение (Direct)".to_string(),
                channel_type: "Direct".to_string(),
                is_available: false,
                latency_ms: 0,
                http_status: None,
                status_text: "FAILED".to_string(),
                details_ru: format!("Ошибка сборки прямого сокета: {}", e),
            };
        }
    };

    match client.get(url).send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let latency = start.elapsed().as_millis() as u64;
            let is_ok = status < 400 || status == 401; // 401 is auth boundary, network is fine
            ChannelProbeResult {
                channel_id: "direct".to_string(),
                channel_name: "Прямое соединение (Direct)".to_string(),
                channel_type: "Direct".to_string(),
                is_available: is_ok,
                latency_ms: latency,
                http_status: Some(status),
                status_text: if is_ok { "SUCCESS".to_string() } else { "BLOCKED".to_string() },
                details_ru: if is_ok {
                    format!("Прямой доступ успешен (HTTP {}, {}мс)", status, latency)
                } else {
                    format!("Прямой запрос заблокирован шлюзом (HTTP {})", status)
                },
            }
        }
        Err(e) => ChannelProbeResult {
            channel_id: "direct".to_string(),
            channel_name: "Прямое соединение (Direct)".to_string(),
            channel_type: "Direct".to_string(),
            is_available: false,
            latency_ms: start.elapsed().as_millis() as u64,
            http_status: None,
            status_text: if e.is_timeout() { "TIMEOUT".to_string() } else { "FAILED".to_string() },
            details_ru: format!("Прямое соединение не удалось: {}", e),
        },
    }
}

async fn probe_system_proxy(url: &str) -> ChannelProbeResult {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let mut proxy_server = None;
    let mut proxy_enable = 0u32;

    if let Ok(settings) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings") {
        proxy_enable = settings.get_value("ProxyEnable").unwrap_or(0);
        if let Ok(srv) = settings.get_value::<String, _>("ProxyServer") {
            proxy_server = Some(srv);
        }
    }

    if proxy_enable == 0 || proxy_server.is_none() {
        return ChannelProbeResult {
            channel_id: "wininet".to_string(),
            channel_name: "Системный прокси Windows (WinINet)".to_string(),
            channel_type: "SystemProxy".to_string(),
            is_available: false,
            latency_ms: 0,
            http_status: None,
            status_text: "SKIPPED".to_string(),
            details_ru: "Системный прокси Windows в реестре отключен.".to_string(),
        };
    }

    let srv_raw = proxy_server.unwrap();
    let proxy_url = if !srv_raw.starts_with("http") && !srv_raw.starts_with("socks") {
        format!("http://{}", srv_raw)
    } else {
        srv_raw
    };

    let start = Instant::now();
    let req_proxy = match reqwest::Proxy::all(&proxy_url) {
        Ok(p) => p,
        Err(e) => {
            return ChannelProbeResult {
                channel_id: "wininet".to_string(),
                channel_name: "Системный прокси Windows (WinINet)".to_string(),
                channel_type: "SystemProxy".to_string(),
                is_available: false,
                latency_ms: 0,
                http_status: None,
                status_text: "FAILED".to_string(),
                details_ru: format!("Неверный адрес системного прокси '{}': {}", proxy_url, e),
            };
        }
    };

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(4000))
        .proxy(req_proxy)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return ChannelProbeResult {
                channel_id: "wininet".to_string(),
                channel_name: "Системный прокси Windows (WinINet)".to_string(),
                channel_type: "SystemProxy".to_string(),
                is_available: false,
                latency_ms: 0,
                http_status: None,
                status_text: "FAILED".to_string(),
                details_ru: format!("Ошибка конфигурации: {}", e),
            };
        }
    };

    match client.get(url).send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let latency = start.elapsed().as_millis() as u64;
            let is_ok = status < 400 || status == 401;
            ChannelProbeResult {
                channel_id: "wininet".to_string(),
                channel_name: "Системный прокси Windows (WinINet)".to_string(),
                channel_type: "SystemProxy".to_string(),
                is_available: is_ok,
                latency_ms: latency,
                http_status: Some(status),
                status_text: if is_ok { "SUCCESS".to_string() } else { "BLOCKED".to_string() },
                details_ru: format!("Туннелирование через '{}' успешно (HTTP {}, {}мс)", proxy_url, status, latency),
            }
        }
        Err(e) => ChannelProbeResult {
            channel_id: "wininet".to_string(),
            channel_name: "Системный прокси Windows (WinINet)".to_string(),
            channel_type: "SystemProxy".to_string(),
            is_available: false,
            latency_ms: start.elapsed().as_millis() as u64,
            http_status: None,
            status_text: "FAILED".to_string(),
            details_ru: format!("Прокси '{}' отклонил запрос: {}", proxy_url, e),
        },
    }
}

async fn probe_env_proxy(url: &str) -> ChannelProbeResult {
    let env_val = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("https_proxy"))
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .or_else(|_| std::env::var("http_proxy"))
        .ok();

    let proxy_url = match env_val {
        Some(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => {
            return ChannelProbeResult {
                channel_id: "env_proxy".to_string(),
                channel_name: "Переменные окружения (HTTP_PROXY)".to_string(),
                channel_type: "EnvProxy".to_string(),
                is_available: false,
                latency_ms: 0,
                http_status: None,
                status_text: "SKIPPED".to_string(),
                details_ru: "Переменные окружения HTTP_PROXY/HTTPS_PROXY не установлены.".to_string(),
            };
        }
    };

    let start = Instant::now();
    let req_proxy = match reqwest::Proxy::all(&proxy_url) {
        Ok(p) => p,
        Err(e) => {
            return ChannelProbeResult {
                channel_id: "env_proxy".to_string(),
                channel_name: "Переменные окружения (HTTP_PROXY)".to_string(),
                channel_type: "EnvProxy".to_string(),
                is_available: false,
                latency_ms: 0,
                http_status: None,
                status_text: "FAILED".to_string(),
                details_ru: format!("Неверный адрес переменной прокси: {}", e),
            };
        }
    };

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(4000))
        .proxy(req_proxy)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return ChannelProbeResult {
                channel_id: "env_proxy".to_string(),
                channel_name: "Переменные окружения (HTTP_PROXY)".to_string(),
                channel_type: "EnvProxy".to_string(),
                is_available: false,
                latency_ms: 0,
                http_status: None,
                status_text: "FAILED".to_string(),
                details_ru: format!("Ошибка HTTP клиента: {}", e),
            };
        }
    };

    match client.get(url).send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let latency = start.elapsed().as_millis() as u64;
            let is_ok = status < 400 || status == 401;
            ChannelProbeResult {
                channel_id: "env_proxy".to_string(),
                channel_name: "Переменные окружения (HTTP_PROXY)".to_string(),
                channel_type: "EnvProxy".to_string(),
                is_available: is_ok,
                latency_ms: latency,
                http_status: Some(status),
                status_text: if is_ok { "SUCCESS".to_string() } else { "FAILED".to_string() },
                details_ru: format!("Переменная '{}' ответила HTTP {} ({}мс)", proxy_url, status, latency),
            }
        }
        Err(e) => ChannelProbeResult {
            channel_id: "env_proxy".to_string(),
            channel_name: "Переменные окружения (HTTP_PROXY)".to_string(),
            channel_type: "EnvProxy".to_string(),
            is_available: false,
            latency_ms: start.elapsed().as_millis() as u64,
            http_status: None,
            status_text: "FAILED".to_string(),
            details_ru: format!("Переменная '{}' вернула ошибку: {}", proxy_url, e),
        },
    }
}

async fn probe_doh(domain: &str) -> ChannelProbeResult {
    let start = Instant::now();
    let doh_url = format!("https://1.1.1.1/dns-query?name={}&type=A", domain);

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(3000))
        .no_proxy()
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return ChannelProbeResult {
                channel_id: "doh".to_string(),
                channel_name: "Защищенный DNS (Cloudflare DoH)".to_string(),
                channel_type: "DoH".to_string(),
                is_available: false,
                latency_ms: 0,
                http_status: None,
                status_text: "FAILED".to_string(),
                details_ru: format!("Ошибка DoH клиента: {}", e),
            };
        }
    };

    match client.get(&doh_url).header("accept", "application/dns-json").send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let latency = start.elapsed().as_millis() as u64;
            let is_ok = status == 200;
            ChannelProbeResult {
                channel_id: "doh".to_string(),
                channel_name: "Защищенный DNS (Cloudflare DoH)".to_string(),
                channel_type: "DoH".to_string(),
                is_available: is_ok,
                latency_ms: latency,
                http_status: Some(status),
                status_text: if is_ok { "SUCCESS".to_string() } else { "FAILED".to_string() },
                details_ru: format!("Cloudflare DoH вернул ответ за {}мс (HTTP {})", latency, status),
            }
        }
        Err(e) => ChannelProbeResult {
            channel_id: "doh".to_string(),
            channel_name: "Защищенный DNS (Cloudflare DoH)".to_string(),
            channel_type: "DoH".to_string(),
            is_available: false,
            latency_ms: start.elapsed().as_millis() as u64,
            http_status: None,
            status_text: "FAILED".to_string(),
            details_ru: format!("Cloudflare DoH недоступен: {}", e),
        },
    }
}

async fn probe_vpn_route() -> ChannelProbeResult {
    let vpn_diag = crate::diag::vpn::diagnose_vpn();
    if vpn_diag.vpn_tunnel_detected {
        let adapter_names: Vec<String> = vpn_diag.active_virtual_adapters.iter().map(|a| a.name.clone()).collect();
        ChannelProbeResult {
            channel_id: "vpn".to_string(),
            channel_name: "Виртуальный туннель (VPN / TUN)".to_string(),
            channel_type: "VPN".to_string(),
            is_available: true,
            latency_ms: 1,
            http_status: None,
            status_text: "SUCCESS".to_string(),
            details_ru: format!("Активен виртуальный сетевой туннель: {}", adapter_names.join(", ")),
        }
    } else {
        ChannelProbeResult {
            channel_id: "vpn".to_string(),
            channel_name: "Виртуальный туннель (VPN / TUN)".to_string(),
            channel_type: "VPN".to_string(),
            is_available: false,
            latency_ms: 0,
            http_status: None,
            status_text: "SKIPPED".to_string(),
            details_ru: "Активных виртуальных адаптеров VPN/TUN в системе не обнаружено.".to_string(),
        }
    }
}
