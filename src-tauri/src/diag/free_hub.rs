use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeProviderInfo {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description_ru: String,
    pub primary_endpoint: String,
    pub base_url: String,
    pub docs_url: String,
    pub is_free_tier: bool,
    pub popular_models: Vec<String>,
    pub recommended_for: String,
    pub direct_rtt_ms: Option<u64>,
    pub http_status: Option<u16>,
    pub is_accessible: bool,
    pub status_text: String,
    pub auth_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicDohInfo {
    pub name: String,
    pub provider: String,
    pub doh_url: String,
    pub primary_ip: String,
    pub secondary_ip: String,
    pub description_ru: String,
    pub features: Vec<String>,
    pub is_tested_ok: bool,
    pub rtt_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeHubReport {
    pub providers: Vec<FreeProviderInfo>,
    pub doh_resolvers: Vec<PublicDohInfo>,
    pub timestamp: String,
}

async fn probe_free_url(client: &reqwest::Client, url: &str) -> (Option<u64>, Option<u16>, bool, String) {
    let start = Instant::now();
    match client.get(url).send().await {
        Ok(res) => {
            let rtt = start.elapsed().as_millis() as u64;
            let status = res.status().as_u16();
            // HTTP 200, 400, 401, 403, 404, 405 all indicate that the IP/DNS/TLS layer reached the server
            // 401/403 means "Server reached, authentication required" which confirms network accessibility
            let accessible = status < 500 && status != 403;
            let text = if status == 200 {
                "Доступен (HTTP 200 OK)".to_string()
            } else if status == 401 {
                "Эндпоинт активен (HTTP 401 Auth Boundary)".to_string()
            } else if status == 403 {
                "Ограничен (HTTP 403 Forbidden / Cloudflare)".to_string()
            } else {
                format!("Ответ сервера HTTP {}", status)
            };
            (Some(rtt), Some(status), accessible || status == 401, text)
        }
        Err(e) => {
            let err_str = e.to_string();
            let summary = if e.is_timeout() {
                "Превышено время ожидания (Timeout)".to_string()
            } else if err_str.contains("connect") || err_str.contains("refused") {
                "Соединение отклонено или недоступно".to_string()
            } else {
                "Сетевая ошибка / блокировка".to_string()
            };
            (None, None, false, summary)
        }
    }
}

pub async fn get_free_hub_data() -> FreeHubReport {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(3500))
        .danger_accept_invalid_certs(false)
        .build()
        .unwrap_or_default();

    // 1. Providers definition
    let mut providers = vec![
        FreeProviderInfo {
            id: "groq".to_string(),
            name: "Groq Cloud Fast Inference".to_string(),
            category: "Высокоскоростной Cloud API".to_string(),
            description_ru: "Сверхбыстрый запуск моделей (до 500 токенов/сек на LPU-процессорах). Щедрый бесплатный тариф с поддержкой открытых весов.".to_string(),
            primary_endpoint: "https://api.groq.com/openai/v1/models".to_string(),
            base_url: "https://api.groq.com/openai/v1".to_string(),
            docs_url: "https://console.groq.com/keys".to_string(),
            is_free_tier: true,
            popular_models: vec![
                "llama-3.3-70b-versatile".to_string(),
                "deepseek-r1-distill-llama-70b".to_string(),
                "gemma2-9b-it".to_string(),
            ],
            recommended_for: "Cursor, VS Code, Zed, быстрая генерация кода".to_string(),
            direct_rtt_ms: None,
            http_status: None,
            is_accessible: false,
            status_text: "Проверка...".to_string(),
            auth_note: "Бесплатный API-ключ в консоли Groq (без привязки карты)".to_string(),
        },
        FreeProviderInfo {
            id: "openrouter".to_string(),
            name: "OpenRouter Free Models Hub".to_string(),
            category: "Универсальный шлюз".to_string(),
            description_ru: "Единый OpenAI-совместимый API с доступом к десяткам полностью бесплатных моделей с суффиксом ':free'.".to_string(),
            primary_endpoint: "https://openrouter.ai/api/v1/models".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            docs_url: "https://openrouter.ai/keys".to_string(),
            is_free_tier: true,
            popular_models: vec![
                "deepseek/deepseek-r1:free".to_string(),
                "meta-llama/llama-3.3-70b-instruct:free".to_string(),
                "google/gemini-2.0-flash-exp:free".to_string(),
            ],
            recommended_for: "Любые IDE и плагины (Continue, Cline, Roo Code, Windsurf)".to_string(),
            direct_rtt_ms: None,
            http_status: None,
            is_accessible: false,
            status_text: "Проверка...".to_string(),
            auth_note: "Бесплатный аккаунт OpenRouter".to_string(),
        },
        FreeProviderInfo {
            id: "deepseek".to_string(),
            name: "DeepSeek Official API".to_string(),
            category: "Прямой API вендора".to_string(),
            description_ru: "Официальный эндпоинт флагманских моделей DeepSeek-V3 и DeepSeek-R1. Высочайшее качество кода при минимальной стоимости.".to_string(),
            primary_endpoint: "https://api.deepseek.com/models".to_string(),
            base_url: "https://api.deepseek.com".to_string(),
            docs_url: "https://platform.deepseek.com/api_keys".to_string(),
            is_free_tier: true,
            popular_models: vec![
                "deepseek-chat (V3)".to_string(),
                "deepseek-reasoner (R1)".to_string(),
            ],
            recommended_for: "Сложный рефакторинг, архитектура, системное мышление".to_string(),
            direct_rtt_ms: None,
            http_status: None,
            is_accessible: false,
            status_text: "Проверка...".to_string(),
            auth_note: "Приветственный грант токенов при регистрации".to_string(),
        },
        FreeProviderInfo {
            id: "mistral".to_string(),
            name: "Mistral AI Le Chat & API".to_string(),
            category: "Европейский ИИ-провайдер".to_string(),
            description_ru: "Открытые европейские модели серии Mistral и Codestral с бесплатным доступом к базовому тиру разработчика.".to_string(),
            primary_endpoint: "https://api.mistral.ai/v1/models".to_string(),
            base_url: "https://api.mistral.ai/v1".to_string(),
            docs_url: "https://console.mistral.ai/api-keys".to_string(),
            is_free_tier: true,
            popular_models: vec![
                "codestral-latest".to_string(),
                "mistral-small-latest".to_string(),
                "open-mistral-nemo".to_string(),
            ],
            recommended_for: "Генерация кода, автодополнение, компактные агенты".to_string(),
            direct_rtt_ms: None,
            http_status: None,
            is_accessible: false,
            status_text: "Проверка...".to_string(),
            auth_note: "Бесплатный tier 'Experiment' на console.mistral.ai".to_string(),
        },
        FreeProviderInfo {
            id: "huggingface".to_string(),
            name: "Hugging Face Serverless Inference".to_string(),
            category: "Открытая экосистема".to_string(),
            description_ru: "Бесплатный запуск тысяч моделей из крупнейшего мирового репозитория машинного обучения без аренды серверов.".to_string(),
            primary_endpoint: "https://api-inference.huggingface.co/models".to_string(),
            base_url: "https://api-inference.huggingface.co/v1".to_string(),
            docs_url: "https://huggingface.co/settings/tokens".to_string(),
            is_free_tier: true,
            popular_models: vec![
                "Qwen/Qwen2.5-Coder-32B-Instruct".to_string(),
                "meta-llama/Llama-3.2-3B-Instruct".to_string(),
            ],
            recommended_for: "Эксперименты с новейшими open-source весами".to_string(),
            direct_rtt_ms: None,
            http_status: None,
            is_accessible: false,
            status_text: "Проверка...".to_string(),
            auth_note: "Бесплатный токен Hugging Face (Read Access)".to_string(),
        },
        FreeProviderInfo {
            id: "ollama_local".to_string(),
            name: "Ollama (Локальный ИИ на ПК)".to_string(),
            category: "Локальный запуск без интернета".to_string(),
            description_ru: "100% приватный и бесплатный запуск моделей прямо на вашем компьютере. Не зависит от провайдеров, блокировок и ключей.".to_string(),
            primary_endpoint: "http://127.0.0.1:11434/api/tags".to_string(),
            base_url: "http://localhost:11434/v1".to_string(),
            docs_url: "https://ollama.com/download".to_string(),
            is_free_tier: true,
            popular_models: vec![
                "qwen2.5-coder:7b".to_string(),
                "deepseek-r1:8b".to_string(),
                "llama3.2:3b".to_string(),
            ],
            recommended_for: "Полная автономность, конфиденциальный код без передачи в сеть".to_string(),
            direct_rtt_ms: None,
            http_status: None,
            is_accessible: false,
            status_text: "Проверка...".to_string(),
            auth_note: "Не требуется авторизация (локальный порт 11434)".to_string(),
        },
        FreeProviderInfo {
            id: "lmstudio_local".to_string(),
            name: "LM Studio (Локальный сервер)".to_string(),
            category: "Локальный запуск без интернета".to_string(),
            description_ru: "Удобный графический интерфейс для запуска моделей GGUF на видеокартах Nvidia/AMD с эмуляцией OpenAI-сервера на порту 1234.".to_string(),
            primary_endpoint: "http://127.0.0.1:1234/v1/models".to_string(),
            base_url: "http://localhost:1234/v1".to_string(),
            docs_url: "https://lmstudio.ai".to_string(),
            is_free_tier: true,
            popular_models: vec![
                "Qwen-2.5-Coder-7B-Instruct-GGUF".to_string(),
                "DeepSeek-R1-Distill-Qwen-8B-GGUF".to_string(),
            ],
            recommended_for: "Локальная разработка с GPU-ускорением".to_string(),
            direct_rtt_ms: None,
            http_status: None,
            is_accessible: false,
            status_text: "Проверка...".to_string(),
            auth_note: "Не требуется авторизация (локальный порт 1234)".to_string(),
        },
    ];

    // Probe all providers in parallel using tokio tasks
    let mut probe_handles = Vec::new();
    for p in &providers {
        let ep = p.primary_endpoint.clone();
        let c = client.clone();
        probe_handles.push(tokio::spawn(async move {
            probe_free_url(&c, &ep).await
        }));
    }

    for (i, handle) in probe_handles.into_iter().enumerate() {
        if let Ok(res) = handle.await {
            providers[i].direct_rtt_ms = res.0;
            providers[i].http_status = res.1;
            providers[i].is_accessible = res.2;
            providers[i].status_text = res.3;
        }
    }

    // 2. Public DoH resolvers definition and probe
    let mut doh_resolvers = vec![
        PublicDohInfo {
            name: "Cloudflare 1.1.1.1".to_string(),
            provider: "Cloudflare DNS".to_string(),
            doh_url: "https://1.1.1.1/dns-query".to_string(),
            primary_ip: "1.1.1.1".to_string(),
            secondary_ip: "1.0.0.1".to_string(),
            description_ru: "Самый быстрый публичный DNS в мире. Шифрование DoH защищает запросы от перехвата и подмены интернет-провайдером.".to_string(),
            features: vec!["DNS-over-HTTPS".to_string(), "Zero Logs".to_string(), "Anycast CDN".to_string()],
            is_tested_ok: false,
            rtt_ms: None,
        },
        PublicDohInfo {
            name: "Google Public DNS".to_string(),
            provider: "Google".to_string(),
            doh_url: "https://dns.google/dns-query".to_string(),
            primary_ip: "8.8.8.8".to_string(),
            secondary_ip: "8.8.4.4".to_string(),
            description_ru: "Глобальная распределенная инфраструктура Google. Отличная устойчивость и независимость от локальных блокировок имен.".to_string(),
            features: vec!["DNS-over-HTTPS".to_string(), "Global GeoDNS".to_string(), "DNSSEC".to_string()],
            is_tested_ok: false,
            rtt_ms: None,
        },
        PublicDohInfo {
            name: "Quad9 Privacy DNS".to_string(),
            provider: "Quad9 Foundation (Швейцария)".to_string(),
            doh_url: "https://dns.quad9.net/dns-query".to_string(),
            primary_ip: "9.9.9.9".to_string(),
            secondary_ip: "149.112.112.112".to_string(),
            description_ru: "Швейцарский некоммерческий резолвер с акцентом на приватность и блокировку фишинговых доменов без цензуры легальных сервисов.".to_string(),
            features: vec!["Строгая приватность (GDPR)".to_string(), "Защита от вредоносных IP".to_string()],
            is_tested_ok: false,
            rtt_ms: None,
        },
        PublicDohInfo {
            name: "AdGuard Public DNS".to_string(),
            provider: "AdGuard".to_string(),
            doh_url: "https://dns.adguard.com/dns-query".to_string(),
            primary_ip: "94.140.14.14".to_string(),
            secondary_ip: "94.140.15.15".to_string(),
            description_ru: "Популярный резолвер с серверами в Европе и СНГ, фильтрующий трекеры и рекламу на уровне запросов.".to_string(),
            features: vec!["DNS-over-HTTPS".to_string(), "Блокировка рекламы".to_string(), "Серверы в СНГ".to_string()],
            is_tested_ok: false,
            rtt_ms: None,
        },
    ];

    // Probe DoH endpoints in parallel
    let mut doh_handles = Vec::new();
    for doh in &doh_resolvers {
        let doh_test_url = format!("{}?name=api.openai.com&type=A", doh.doh_url);
        let c = client.clone();
        doh_handles.push(tokio::spawn(async move {
            let start = Instant::now();
            match c.get(&doh_test_url).header("accept", "application/dns-json").send().await {
                Ok(res) => {
                    let rtt = start.elapsed().as_millis() as u64;
                    (res.status().is_success(), Some(rtt))
                }
                Err(_) => (false, None),
            }
        }));
    }

    for (i, handle) in doh_handles.into_iter().enumerate() {
        if let Ok(res) = handle.await {
            doh_resolvers[i].is_tested_ok = res.0;
            doh_resolvers[i].rtt_ms = res.1;
        }
    }

    FreeHubReport {
        providers,
        doh_resolvers,
        timestamp: Utc::now().to_rfc3339(),
    }
}
