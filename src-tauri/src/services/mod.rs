use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceStatusKind {
    Pass,
    AuthBoundary,
    RateLimited,
    DnsFailure,
    TcpFailure,
    TlsFailure,
    ProxyFailure,
    GeoBlocked,
    ServerFailure,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointCheck {
    pub url: String,
    pub host: String,
    pub dns_ok: bool,
    pub tcp_ok: bool,
    pub tls_ok: bool,
    pub http_status: Option<u16>,
    pub latency_ms: u64,
    pub server_header: Option<String>,
    pub status_kind: ServiceStatusKind,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceProfileResult {
    pub id: String,
    pub name: String,
    pub category: String,
    pub overall_status: String, // "pass" | "warning" | "fail" | "unknown"
    pub status_kind: ServiceStatusKind,
    pub primary_endpoint: String,
    pub endpoints: Vec<EndpointCheck>,
    pub latency_ms: u64,
    pub summary_ru: String,
    pub summary_en: String,
    pub recommendation_ru: Option<String>,
}

/// Perform fine-grained layered probe of an individual HTTPS endpoint
pub async fn probe_endpoint(url_str: &str, proxy_override: Option<&str>) -> EndpointCheck {
    let start = Instant::now();
    let parsed_url = match reqwest::Url::parse(url_str) {
        Ok(u) => u,
        Err(e) => {
            return EndpointCheck {
                url: url_str.to_string(),
                host: "unknown".to_string(),
                dns_ok: false,
                tcp_ok: false,
                tls_ok: false,
                http_status: None,
                latency_ms: 0,
                server_header: None,
                status_kind: ServiceStatusKind::Unknown,
                details: format!("Invalid URL: {}", e),
            };
        }
    };

    let host = parsed_url.host_str().unwrap_or("").to_string();
    let port = parsed_url.port_or_known_default().unwrap_or(443);

    // 1. DNS check
    let dns_ok = match tokio::net::lookup_host((host.as_str(), port)).await {
        Ok(mut addrs) => addrs.next().is_some(),
        Err(_) => false,
    };

    if !dns_ok {
        return EndpointCheck {
            url: url_str.to_string(),
            host: host.clone(),
            dns_ok: false,
            tcp_ok: false,
            tls_ok: false,
            http_status: None,
            latency_ms: start.elapsed().as_millis() as u64,
            server_header: None,
            status_kind: ServiceStatusKind::DnsFailure,
            details: format!("Не удалось разрешить DNS-имя хоста {}", host),
        };
    }

    // 2. TCP connect check
    let tcp_conn = tokio::time::timeout(
        Duration::from_millis(3000),
        tokio::net::TcpStream::connect(format!("{}:{}", host, port)),
    )
    .await;

    let tcp_ok = match tcp_conn {
        Ok(Ok(_)) => true,
        _ => false,
    };

    if !tcp_ok {
        return EndpointCheck {
            url: url_str.to_string(),
            host: host.clone(),
            dns_ok: true,
            tcp_ok: false,
            tls_ok: false,
            http_status: None,
            latency_ms: start.elapsed().as_millis() as u64,
            server_header: None,
            status_kind: ServiceStatusKind::TcpFailure,
            details: format!("TCP соединение с {}:{} было сброшено или отклонено таймаутом.", host, port),
        };
    }

    // 3. HTTP Client execution (with proxy if configured)
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_millis(4000))
        .danger_accept_invalid_certs(false);

    if let Some(proxy_url) = proxy_override {
        if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
            builder = builder.proxy(proxy);
        }
    }

    let client = match builder.build() {
        Ok(c) => c,
        Err(e) => {
            return EndpointCheck {
                url: url_str.to_string(),
                host,
                dns_ok: true,
                tcp_ok: true,
                tls_ok: false,
                http_status: None,
                latency_ms: start.elapsed().as_millis() as u64,
                server_header: None,
                status_kind: ServiceStatusKind::Unknown,
                details: format!("Ошибка конфигурации HTTP клиента: {}", e),
            };
        }
    };

    let req_res = client.get(url_str).header("User-Agent", "AIAccessDoctor/1.0").send().await;
    let latency_ms = start.elapsed().as_millis() as u64;

    match req_res {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let server_hdr = resp.headers().get("server").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
            let body_sample = resp.text().await.unwrap_or_default();
            let body_lower = body_sample.to_lowercase();

            // Analyze HTTP status and headers
            let (status_kind, details) = if status == 200 || status == 204 {
                (
                    ServiceStatusKind::Pass,
                    format!("Сервис полностью доступен (HTTP {}, ответ за {}мс)", status, latency_ms),
                )
            } else if status == 401 || (status == 403 && (body_lower.contains("api key") || body_lower.contains("unauthorized") || body_lower.contains("authentication") || body_lower.contains("credentials"))) {
                (
                    ServiceStatusKind::AuthBoundary,
                    format!("Сервис успешно отвечает на сетевом уровне (HTTP {} Auth Boundary). Требуется авторизация или ключ API.", status),
                )
            } else if status == 403 && (body_lower.contains("country") || body_lower.contains("region") || body_lower.contains("unsupported_country") || body_lower.contains("geo")) || status == 451 {
                (
                    ServiceStatusKind::GeoBlocked,
                    format!("Доступ заблокирован по географическому признаку (HTTP {}, Geo-Restricted).", status),
                )
            } else if status == 429 {
                (
                    ServiceStatusKind::RateLimited,
                    "Превышен лимит запросов (HTTP 429 Too Many Requests).".to_string(),
                )
            } else if status == 502 || status == 504 {
                if body_lower.contains("proxy") || server_hdr.as_deref().unwrap_or("").contains("squid") {
                    (
                        ServiceStatusKind::ProxyFailure,
                        format!("Прокси-сервер вернул HTTP {} (шлюз недоступен). Проверьте настройки прокси.", status),
                    )
                } else {
                    (
                        ServiceStatusKind::ServerFailure,
                        format!("Внутренний сбой на стороне сервера ИИ-сервиса (HTTP {}).", status),
                    )
                }
            } else if status >= 500 {
                (
                    ServiceStatusKind::ServerFailure,
                    format!("Сервер вернул код ошибки {} (Server Error).", status),
                )
            } else {
                (
                    ServiceStatusKind::Pass,
                    format!("Эндпоинт ответил кодом HTTP {}", status),
                )
            };

            EndpointCheck {
                url: url_str.to_string(),
                host,
                dns_ok: true,
                tcp_ok: true,
                tls_ok: true,
                http_status: Some(status),
                latency_ms,
                server_header: server_hdr,
                status_kind,
                details,
            }
        }
        Err(e) => {
            let err_str = e.to_string();
            let (status_kind, details) = if err_str.contains("certificate") || err_str.contains("tls") || err_str.contains("ssl") {
                (
                    ServiceStatusKind::TlsFailure,
                    format!("Сбой проверки сертификата TLS: {}", err_str),
                )
            } else if err_str.contains("proxy") || err_str.contains("502") {
                (
                    ServiceStatusKind::ProxyFailure,
                    format!("Ошибка туннелирования через прокси: {}", err_str),
                )
            } else if e.is_timeout() {
                (
                    ServiceStatusKind::TcpFailure,
                    format!("Таймаут соединения с эндпоинтом: {}", err_str),
                )
            } else {
                (
                    ServiceStatusKind::Unknown,
                    format!("Сетевой сбой при отправке запроса: {}", err_str),
                )
            };

            EndpointCheck {
                url: url_str.to_string(),
                host,
                dns_ok: true,
                tcp_ok: true,
                tls_ok: false,
                http_status: None,
                latency_ms,
                server_header: None,
                status_kind,
                details,
            }
        }
    }
}

/// Diagnose Google Antigravity service
pub async fn diagnose_antigravity(proxy_override: Option<&str>) -> ServiceProfileResult {
    let endpoints = vec![
        "https://accounts.google.com",
        "https://cloudcode.googleapis.com",
        "https://oauth2.googleapis.com",
    ];

    let mut checks = Vec::new();
    for ep in endpoints {
        checks.push(probe_endpoint(ep, proxy_override).await);
    }

    evaluate_profile(
        "antigravity",
        "Google Antigravity",
        "Google Cloud & AI Code Engine",
        checks,
    )
}

/// Diagnose Google Gemini service
pub async fn diagnose_gemini(proxy_override: Option<&str>) -> ServiceProfileResult {
    let endpoints = vec![
        "https://generativelanguage.googleapis.com",
        "https://aiplatform.googleapis.com",
    ];

    let mut checks = Vec::new();
    for ep in endpoints {
        checks.push(probe_endpoint(ep, proxy_override).await);
    }

    evaluate_profile(
        "gemini",
        "Google Gemini",
        "Large Language Models & Multimodal API",
        checks,
    )
}

/// Diagnose OpenAI / Codex service
pub async fn diagnose_openai(proxy_override: Option<&str>) -> ServiceProfileResult {
    let endpoints = vec![
        "https://api.openai.com/v1/models",
        "https://chatgpt.com",
        "https://auth.openai.com",
    ];

    let mut checks = Vec::new();
    for ep in endpoints {
        checks.push(probe_endpoint(ep, proxy_override).await);
    }

    evaluate_profile(
        "openai",
        "OpenAI / Codex",
        "GPT-4o, Codex & Embeddings API",
        checks,
    )
}

/// Diagnose Anthropic Claude service
pub async fn diagnose_claude(proxy_override: Option<&str>) -> ServiceProfileResult {
    let endpoints = vec![
        "https://api.anthropic.com/v1/messages",
        "https://claude.ai",
    ];

    let mut checks = Vec::new();
    for ep in endpoints {
        checks.push(probe_endpoint(ep, proxy_override).await);
    }

    evaluate_profile(
        "claude",
        "Anthropic Claude",
        "Claude 3.5 Sonnet & Haiku API",
        checks,
    )
}

/// Diagnose GitHub Developer Services & Copilot
pub async fn diagnose_github(proxy_override: Option<&str>) -> ServiceProfileResult {
    let endpoints = vec![
        "https://api.github.com",
        "https://github.com",
        "https://copilot-proxy.githubusercontent.com",
    ];

    let mut checks = Vec::new();
    for ep in endpoints {
        checks.push(probe_endpoint(ep, proxy_override).await);
    }

    evaluate_profile(
        "github",
        "GitHub Developer Services",
        "Git Repositories, REST API & Copilot Gateway",
        checks,
    )
}

/// Aggregator and evaluator for service profile results
fn evaluate_profile(
    id: &str,
    name: &str,
    category: &str,
    checks: Vec<EndpointCheck>,
) -> ServiceProfileResult {
    let primary_endpoint = checks.first().map(|c| c.url.clone()).unwrap_or_default();
    let total_latency: u64 = checks.iter().map(|c| c.latency_ms).sum();
    let avg_latency = if !checks.is_empty() { total_latency / checks.len() as u64 } else { 0 };

    // Worst status precedence: ProxyFailure > DnsFailure > TcpFailure > TlsFailure > GeoBlocked > RateLimited > ServerFailure > AuthBoundary > Pass
    let mut dominant_kind = ServiceStatusKind::Pass;
    for c in &checks {
        match c.status_kind {
            ServiceStatusKind::ProxyFailure => {
                dominant_kind = ServiceStatusKind::ProxyFailure;
                break;
            }
            ServiceStatusKind::DnsFailure => {
                dominant_kind = ServiceStatusKind::DnsFailure;
            }
            ServiceStatusKind::TcpFailure if dominant_kind != ServiceStatusKind::DnsFailure => {
                dominant_kind = ServiceStatusKind::TcpFailure;
            }
            ServiceStatusKind::TlsFailure if dominant_kind != ServiceStatusKind::DnsFailure && dominant_kind != ServiceStatusKind::TcpFailure => {
                dominant_kind = ServiceStatusKind::TlsFailure;
            }
            ServiceStatusKind::GeoBlocked if dominant_kind == ServiceStatusKind::Pass || dominant_kind == ServiceStatusKind::AuthBoundary => {
                dominant_kind = ServiceStatusKind::GeoBlocked;
            }
            ServiceStatusKind::RateLimited if dominant_kind == ServiceStatusKind::Pass || dominant_kind == ServiceStatusKind::AuthBoundary => {
                dominant_kind = ServiceStatusKind::RateLimited;
            }
            ServiceStatusKind::ServerFailure if dominant_kind == ServiceStatusKind::Pass || dominant_kind == ServiceStatusKind::AuthBoundary => {
                dominant_kind = ServiceStatusKind::ServerFailure;
            }
            ServiceStatusKind::AuthBoundary if dominant_kind == ServiceStatusKind::Pass => {
                dominant_kind = ServiceStatusKind::AuthBoundary;
            }
            _ => {}
        }
    }

    let (overall_status, summary_ru, summary_en, recommendation_ru) = match dominant_kind {
        ServiceStatusKind::Pass => (
            "pass".to_string(),
            "Сервис полностью доступен. Сетевой канал чист, задержка оптимальна.".to_string(),
            "Service is fully reachable. Network path is clear, latency is optimal.".to_string(),
            None,
        ),
        ServiceStatusKind::AuthBoundary => (
            "pass".to_string(),
            "Сервис доступен на сетевом уровне. Требуется передача ключа API или вход в аккаунт.".to_string(),
            "Service is reachable at network level. Valid API key or account login required.".to_string(),
            Some("Сетевых блокировок нет. Если инструмент разработчика выдаёт ошибку, проверьте валидность API-ключа в конфигурационном файле.".to_string()),
        ),
        ServiceStatusKind::ProxyFailure => (
            "fail".to_string(),
            "Сбой соединения через локальный прокси-сервер (Bad Gateway / Dead Port).".to_string(),
            "Proxy forwarding failure (Bad Gateway or dead port).".to_string(),
            Some("Проверьте статус локального прокси-клиента (v2ray, Xray, sing-box) или сбросьте переменные HTTP_PROXY.".to_string()),
        ),
        ServiceStatusKind::GeoBlocked => (
            "fail".to_string(),
            "Доступ заблокирован сервисом по географическому региону вашего IP.".to_string(),
            "Access blocked due to geographical IP restriction.".to_string(),
            Some("Используйте VPN или прокси с выходом в поддерживаемом регионе (США, Европа).".to_string()),
        ),
        ServiceStatusKind::DnsFailure => (
            "fail".to_string(),
            "Не удается разрешить доменное имя сервиса через DNS.".to_string(),
            "Cannot resolve service domain name via DNS.".to_string(),
            Some("Смените DNS-сервер в свойствах сетевого адаптера на 1.1.1.1 или 8.8.8.8.".to_string()),
        ),
        ServiceStatusKind::TcpFailure => (
            "fail".to_string(),
            "TCP соединение с сервером сброшено или заблокировано брандмауэром/провайдером.".to_string(),
            "TCP connection reset or blocked by ISP/firewall.".to_string(),
            Some("Порт 443 блокируется на пути к серверу. Активируйте VPN или обход блокировок.".to_string()),
        ),
        ServiceStatusKind::TlsFailure => (
            "fail".to_string(),
            "Сбой проверки сертификата безопасности TLS. Возможно вмешательство антивируса или MITM.".to_string(),
            "TLS certificate validation failed. Possible antivirus SSL inspection or MITM middlebox.".to_string(),
            Some("Отключите сканирование HTTPS в настройках антивируса (Kaspersky, ESET) или добавьте CA в доверенные.".to_string()),
        ),
        ServiceStatusKind::RateLimited => (
            "warning".to_string(),
            "Превышен лимит обращений к API (429 Rate Limit).".to_string(),
            "API rate limit exceeded (HTTP 429).".to_string(),
            Some("Подождите некоторое время перед повторными запросами или обновите тарифный план.".to_string()),
        ),
        ServiceStatusKind::ServerFailure => (
            "warning".to_string(),
            "Сбой на стороне поставщика сервиса (HTTP 5xx Server Error).".to_string(),
            "Server-side outage at provider (HTTP 5xx).".to_string(),
            Some("Проверьте официальный статус-бар сервиса (Status Page). Проблема не на вашем ПК.".to_string()),
        ),
        ServiceStatusKind::Unknown => (
            "unknown".to_string(),
            "Неопределённая ошибка при связи с сервисом.".to_string(),
            "Indeterminate communication failure.".to_string(),
            Some("Проведите повторную диагностику или проверьте системный журнал.".to_string()),
        ),
    };

    ServiceProfileResult {
        id: id.to_string(),
        name: name.to_string(),
        category: category.to_string(),
        overall_status,
        status_kind: dominant_kind,
        primary_endpoint,
        endpoints: checks,
        latency_ms: avg_latency,
        summary_ru,
        summary_en,
        recommendation_ru,
    }
}
