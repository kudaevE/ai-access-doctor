use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyValidationReport {
    pub provider: String,
    pub is_valid: bool,
    pub is_geo_blocked: bool,
    pub http_status: u16,
    pub latency_ms: u64,
    pub message: String,
    pub available_models: Vec<String>,
    pub quota_info: Option<String>,
}

pub async fn validate_ai_api_key(
    provider: String,
    api_key: String,
    proxy_url: Option<String>,
) -> Result<ApiKeyValidationReport, String> {
    let trimmed_key = api_key.trim();
    if trimmed_key.is_empty() {
        return Err("API-ключ не может быть пустым.".to_string());
    }

    let mut client_builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(8));

    if let Some(proxy_str) = proxy_url.filter(|p| !p.trim().is_empty()) {
        if let Ok(proxy) = reqwest::Proxy::all(&proxy_str) {
            client_builder = client_builder.proxy(proxy);
        }
    }

    let client = client_builder.build().map_err(|e| format!("Ошибка создания HTTP клиента: {}", e))?;

    let start = Instant::now();
    let norm_provider = provider.to_lowercase();

    match norm_provider.as_str() {
        "gemini" | "google" => {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                trimmed_key
            );
            let resp = client.get(&url).send().await;
            let latency_ms = start.elapsed().as_millis() as u64;

            match resp {
                Ok(res) => {
                    let status = res.status().as_u16();
                    let body = res.text().await.unwrap_or_default();

                    if status == 200 {
                        let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
                        let mut models = Vec::new();
                        if let Some(arr) = json["models"].as_array() {
                            for m in arr.iter().take(8) {
                                if let Some(name) = m["name"].as_str() {
                                    models.push(name.replace("models/", ""));
                                }
                            }
                        }
                        Ok(ApiKeyValidationReport {
                            provider: "Google Gemini".to_string(),
                            is_valid: true,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: format!("Ключ активен! Доступно {} моделей Gemini.", models.len()),
                            available_models: models,
                            quota_info: Some("Тарифный план активен (Google AI Studio)".to_string()),
                        })
                    } else if body.contains("User location is not supported") || body.contains("FAILED_PRECONDITION") {
                        Ok(ApiKeyValidationReport {
                            provider: "Google Gemini".to_string(),
                            is_valid: true,
                            is_geo_blocked: true,
                            http_status: status,
                            latency_ms,
                            message: "Ключ валиден, но ваш IP заблокирован Google (User location is not supported). Используйте SmartDNS или европейский мост AI Access Doctor!".to_string(),
                            available_models: Vec::new(),
                            quota_info: None,
                        })
                    } else if status == 400 || status == 403 {
                        Ok(ApiKeyValidationReport {
                            provider: "Google Gemini".to_string(),
                            is_valid: false,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: "Неверный API-ключ Google Gemini или отозванный доступ.".to_string(),
                            available_models: Vec::new(),
                            quota_info: None,
                        })
                    } else {
                        Ok(ApiKeyValidationReport {
                            provider: "Google Gemini".to_string(),
                            is_valid: false,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: format!("Google API ответил кодом {}: {}", status, body.chars().take(120).collect::<String>()),
                            available_models: Vec::new(),
                            quota_info: None,
                        })
                    }
                }
                Err(e) => Err(format!("Таймаут или сетевой сбой подключения к Gemini API: {}", e)),
            }
        }
        "openai" | "chatgpt" => {
            let url = "https://api.openai.com/v1/models";
            let resp = client
                .get(url)
                .header("Authorization", format!("Bearer {}", trimmed_key))
                .send()
                .await;
            let latency_ms = start.elapsed().as_millis() as u64;

            match resp {
                Ok(res) => {
                    let status = res.status().as_u16();
                    let body = res.text().await.unwrap_or_default();

                    if status == 200 {
                        let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
                        let mut models = Vec::new();
                        if let Some(arr) = json["data"].as_array() {
                            for m in arr {
                                if let Some(id) = m["id"].as_str() {
                                    if id.starts_with("gpt-4") || id.starts_with("o1") || id.starts_with("o3") {
                                        models.push(id.to_string());
                                    }
                                }
                            }
                        }
                        models.truncate(8);
                        Ok(ApiKeyValidationReport {
                            provider: "OpenAI".to_string(),
                            is_valid: true,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: "Ключ OpenAI полностью валиден и готов к запросам!".to_string(),
                            available_models: models,
                            quota_info: Some("Активный аккаунт OpenAI API".to_string()),
                        })
                    } else if status == 401 {
                        Ok(ApiKeyValidationReport {
                            provider: "OpenAI".to_string(),
                            is_valid: false,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: "Неверный ключ OpenAI (Incorrect API key provided).".to_string(),
                            available_models: Vec::new(),
                            quota_info: None,
                        })
                    } else if status == 403 || body.contains("unsupported_country_region_territory") {
                        Ok(ApiKeyValidationReport {
                            provider: "OpenAI".to_string(),
                            is_valid: true,
                            is_geo_blocked: true,
                            http_status: status,
                            latency_ms,
                            message: "Страна заблокирована со стороны Cloudflare/OpenAI. Включите прокси или SmartDNS.".to_string(),
                            available_models: Vec::new(),
                            quota_info: None,
                        })
                    } else if status == 429 {
                        Ok(ApiKeyValidationReport {
                            provider: "OpenAI".to_string(),
                            is_valid: true,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: "Ключ действителен, но исчерпан лимит квоты (Quota Exceeded / Rate Limit).".to_string(),
                            available_models: Vec::new(),
                            quota_info: Some("Превышена квота (429 Too Many Requests)".to_string()),
                        })
                    } else {
                        Ok(ApiKeyValidationReport {
                            provider: "OpenAI".to_string(),
                            is_valid: false,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: format!("Код ответа OpenAI: {}", status),
                            available_models: Vec::new(),
                            quota_info: None,
                        })
                    }
                }
                Err(e) => Err(format!("Сетевая ошибка при проверке OpenAI: {}", e)),
            }
        }
        "anthropic" | "claude" => {
            let url = "https://api.anthropic.com/v1/models";
            let resp = client
                .get(url)
                .header("x-api-key", trimmed_key)
                .header("anthropic-version", "2023-06-01")
                .send()
                .await;
            let latency_ms = start.elapsed().as_millis() as u64;

            match resp {
                Ok(res) => {
                    let status = res.status().as_u16();
                    let body = res.text().await.unwrap_or_default();

                    if status == 200 {
                        let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
                        let mut models = Vec::new();
                        if let Some(arr) = json["data"].as_array() {
                            for m in arr {
                                if let Some(id) = m["id"].as_str() {
                                    models.push(id.to_string());
                                }
                            }
                        }
                        Ok(ApiKeyValidationReport {
                            provider: "Anthropic Claude".to_string(),
                            is_valid: true,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: "Ключ Claude активен! Доступ подтвержден.".to_string(),
                            available_models: models,
                            quota_info: Some("Активный рабочий ключ Claude API".to_string()),
                        })
                    } else if status == 401 {
                        Ok(ApiKeyValidationReport {
                            provider: "Anthropic Claude".to_string(),
                            is_valid: false,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: "Недействительный ключ Anthropic API (401 Unauthorized).".to_string(),
                            available_models: Vec::new(),
                            quota_info: None,
                        })
                    } else if status == 403 || body.contains("restricted_country") {
                        Ok(ApiKeyValidationReport {
                            provider: "Anthropic Claude".to_string(),
                            is_valid: true,
                            is_geo_blocked: true,
                            http_status: status,
                            latency_ms,
                            message: "Регион заблокирован Anthropic. Требуется европейский прокси.".to_string(),
                            available_models: Vec::new(),
                            quota_info: None,
                        })
                    } else {
                        Ok(ApiKeyValidationReport {
                            provider: "Anthropic Claude".to_string(),
                            is_valid: false,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: format!("Ответ Claude API {}: {}", status, body.chars().take(100).collect::<String>()),
                            available_models: Vec::new(),
                            quota_info: None,
                        })
                    }
                }
                Err(e) => Err(format!("Сетевая ошибка проверки Anthropic: {}", e)),
            }
        }
        "groq" => {
            let url = "https://api.groq.com/openai/v1/models";
            let resp = client
                .get(url)
                .header("Authorization", format!("Bearer {}", trimmed_key))
                .send()
                .await;
            let latency_ms = start.elapsed().as_millis() as u64;

            match resp {
                Ok(res) => {
                    let status = res.status().as_u16();
                    let body = res.text().await.unwrap_or_default();

                    if status == 200 {
                        let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
                        let mut models = Vec::new();
                        if let Some(arr) = json["data"].as_array() {
                            for m in arr.iter().take(8) {
                                if let Some(id) = m["id"].as_str() {
                                    models.push(id.to_string());
                                }
                            }
                        }
                        Ok(ApiKeyValidationReport {
                            provider: "Groq LPU".to_string(),
                            is_valid: true,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: format!("Ключ Groq ультра-быстрого инференса активен! ({} мс)", latency_ms),
                            available_models: models,
                            quota_info: Some("Groq Cloud активен (Free/Pro)".to_string()),
                        })
                    } else {
                        Ok(ApiKeyValidationReport {
                            provider: "Groq LPU".to_string(),
                            is_valid: false,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: "Неверный ключ Groq API или исчерпаны квоты.".to_string(),
                            available_models: Vec::new(),
                            quota_info: None,
                        })
                    }
                }
                Err(e) => Err(format!("Сетевая ошибка Groq: {}", e)),
            }
        }
        "openrouter" => {
            let url = "https://openrouter.ai/api/v1/auth/key";
            let resp = client
                .get(url)
                .header("Authorization", format!("Bearer {}", trimmed_key))
                .send()
                .await;
            let latency_ms = start.elapsed().as_millis() as u64;

            match resp {
                Ok(res) => {
                    let status = res.status().as_u16();
                    let body = res.text().await.unwrap_or_default();

                    if status == 200 {
                        let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
                        let data = &json["data"];
                        let label = data["label"].as_str().unwrap_or("API Key");
                        let limit = data["limit"].as_f64();
                        let usage = data["usage"].as_f64().unwrap_or(0.0);

                        let quota_str = if let Some(l) = limit {
                            format!("Использовано: ${:.2} из ${:.2}", usage, l)
                        } else {
                            format!("Использовано: ${:.2} (Безлимитно)", usage)
                        };

                        Ok(ApiKeyValidationReport {
                            provider: "OpenRouter".to_string(),
                            is_valid: true,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: format!("Ключ OpenRouter '{}' валиден! Баланс активен.", label),
                            available_models: vec![
                                "anthropic/claude-3.7-sonnet".to_string(),
                                "google/gemini-2.5-pro".to_string(),
                                "openai/gpt-4o".to_string(),
                                "deepseek/deepseek-chat".to_string(),
                            ],
                            quota_info: Some(quota_str),
                        })
                    } else {
                        Ok(ApiKeyValidationReport {
                            provider: "OpenRouter".to_string(),
                            is_valid: false,
                            is_geo_blocked: false,
                            http_status: status,
                            latency_ms,
                            message: "Недействительный ключ OpenRouter.".to_string(),
                            available_models: Vec::new(),
                            quota_info: None,
                        })
                    }
                }
                Err(e) => Err(format!("Сетевая ошибка OpenRouter: {}", e)),
            }
        }
        _ => Err(format!("Неизвестный провайдер: {}. Поддерживаются: gemini, openai, claude, groq, openrouter", provider)),
    }
}
