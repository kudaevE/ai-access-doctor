use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTargetResult {
    pub id: String,
    pub name: String,
    pub category: String,
    pub endpoint: String,
    pub latency_ms: Option<u64>,
    pub http_status: Option<u16>,
    pub is_available: bool,
    pub speed_rating: String, // "ULTRA_FAST", "FAST", "NORMAL", "SLOW", "BLOCKED"
    pub note_ru: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedBenchmarkReport {
    pub timestamp: String,
    pub fastest_provider_name: Option<String>,
    pub fastest_provider_rtt_ms: Option<u64>,
    pub total_tested: usize,
    pub available_count: usize,
    pub results: Vec<BenchmarkTargetResult>,
    pub recommendation_ru: String,
}

struct TargetDef {
    id: &'static str,
    name: &'static str,
    category: &'static str,
    endpoint: &'static str,
}

pub async fn run_ai_speed_benchmark() -> SpeedBenchmarkReport {
    let targets = vec![
        TargetDef {
            id: "groq",
            name: "Groq Cloud Fast Inference",
            category: "LPU Ultra-Fast",
            endpoint: "https://api.groq.com/openai/v1/models",
        },
        TargetDef {
            id: "gemini",
            name: "Google Gemini 2.0 API",
            category: "Google Cloud",
            endpoint: "https://generativelanguage.googleapis.com",
        },
        TargetDef {
            id: "antigravity",
            name: "Google Antigravity PA",
            category: "Language Server",
            endpoint: "https://cloudcode-pa.googleapis.com",
        },
        TargetDef {
            id: "openrouter",
            name: "OpenRouter Unified API",
            category: "Router 100+ Models",
            endpoint: "https://openrouter.ai/api/v1/models",
        },
        TargetDef {
            id: "cerebras",
            name: "Cerebras Inference Engine",
            category: "Wafer-Scale AI",
            endpoint: "https://api.cerebras.ai/v1/models",
        },
        TargetDef {
            id: "mistral",
            name: "Mistral Codestral API",
            category: "Code Specialist",
            endpoint: "https://api.mistral.ai/v1/models",
        },
        TargetDef {
            id: "openai",
            name: "OpenAI GPT-4o / Codex",
            category: "OpenAI Cloud",
            endpoint: "https://api.openai.com/v1/models",
        },
        TargetDef {
            id: "claude",
            name: "Anthropic Claude 3.5",
            category: "Anthropic Cloud",
            endpoint: "https://api.anthropic.com/v1/messages",
        },
        TargetDef {
            id: "github",
            name: "GitHub Developer Services",
            category: "Copilot & Repos",
            endpoint: "https://api.github.com",
        },
    ];

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(4000))
        .danger_accept_invalid_certs(false)
        .build()
        .unwrap_or_default();

    let mut handles = Vec::new();

    for t in targets {
        let c = client.clone();
        handles.push(tokio::spawn(async move {
            let start = Instant::now();
            match c.get(t.endpoint).send().await {
                Ok(res) => {
                    let rtt = start.elapsed().as_millis() as u64;
                    let status = res.status().as_u16();
                    // 200, 401, 403, 404 all indicate host reached
                    let is_available = status < 500 && status != 403;
                    let speed_rating = if !is_available {
                        "BLOCKED".to_string()
                    } else if rtt < 100 {
                        "ULTRA_FAST".to_string()
                    } else if rtt < 220 {
                        "FAST".to_string()
                    } else if rtt < 450 {
                        "NORMAL".to_string()
                    } else {
                        "SLOW".to_string()
                    };

                    let note_ru = if status == 200 {
                        format!("Ответ 200 OK ({} мс)", rtt)
                    } else if status == 401 {
                        format!("Сервер доступен (Auth Boundary 401, {} мс)", rtt)
                    } else if status == 403 {
                        "Блокировка доступа (HTTP 403 Forbidden / Cloudflare)".to_string()
                    } else {
                        format!("HTTP статус {} ({} мс)", status, rtt)
                    };

                    BenchmarkTargetResult {
                        id: t.id.to_string(),
                        name: t.name.to_string(),
                        category: t.category.to_string(),
                        endpoint: t.endpoint.to_string(),
                        latency_ms: Some(rtt),
                        http_status: Some(status),
                        is_available,
                        speed_rating,
                        note_ru,
                    }
                }
                Err(e) => {
                    let err_str = e.to_string();
                    let note_ru = if e.is_timeout() {
                        "Превышено время ожидания (Таймаут > 4 сек)".to_string()
                    } else if err_str.contains("connect") || err_str.contains("refused") {
                        "Соединение отклонено или недоступно в РФ".to_string()
                    } else {
                        "Сетевая ошибка соединения".to_string()
                    };

                    BenchmarkTargetResult {
                        id: t.id.to_string(),
                        name: t.name.to_string(),
                        category: t.category.to_string(),
                        endpoint: t.endpoint.to_string(),
                        latency_ms: None,
                        http_status: None,
                        is_available: false,
                        speed_rating: "BLOCKED".to_string(),
                        note_ru,
                    }
                }
            }
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(res) = handle.await {
            results.push(res);
        }
    }

    // Sort: available first, then by latency ascending
    results.sort_by(|a, b| {
        match (a.is_available, b.is_available) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.latency_ms.cmp(&b.latency_ms),
        }
    });

    let available_count = results.iter().filter(|r| r.is_available).count();
    let fastest = results.iter().find(|r| r.is_available && r.latency_ms.is_some());

    let (fastest_name, fastest_rtt) = if let Some(f) = fastest {
        (Some(f.name.clone()), f.latency_ms)
    } else {
        (None, None)
    };

    let recommendation_ru = if let (Some(name), Some(rtt)) = (&fastest_name, fastest_rtt) {
        format!(
            "Рекомендуемый провайдер: «{}» с наименьшим временем отклика ({} мс). Подходит для автодополнения кода и диалогов без задержек.",
            name, rtt
        )
    } else {
        "Все прямые каналы заблокированы. Рекомендуется активировать Antigravity Relay или DoH.".to_string()
    };

    SpeedBenchmarkReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        fastest_provider_name: fastest_name,
        fastest_provider_rtt_ms: fastest_rtt,
        total_tested: results.len(),
        available_count,
        results,
        recommendation_ru,
    }
}
