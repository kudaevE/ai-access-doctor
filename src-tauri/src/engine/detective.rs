use crate::storage::db::sanitize_text;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetectiveResult {
    pub category: String,
    pub error_code: String,
    pub title_ru: String,
    pub meaning_ru: String,
    pub confirmed_facts: Vec<String>,
    pub likely_cause_ru: String,
    pub suggested_tests: Vec<String>,
    pub safe_fix_ru: Vec<String>,
    pub sanitized_input: String,
}

pub fn analyze_error_text(raw_input: &str) -> ErrorDetectiveResult {
    let sanitized = sanitize_text(raw_input);
    let lower = sanitized.to_lowercase();

    let mut facts = Vec::new();
    let mut tests = Vec::new();
    let mut fixes = Vec::new();

    // 1. HTTP 401 Unauthorized / Invalid API Key
    if lower.contains("401")
        || lower.contains("unauthorized")
        || lower.contains("incorrect api key")
        || lower.contains("invalid x-api-key")
        || lower.contains("bad credentials")
        || lower.contains("authentication failed")
    {
        facts.push("Сетевое соединение успешно установлено (TCP и TLS рукопожатия пройдены).".to_string());
        facts.push("Сервер ИИ отклонил переданный токен авторизации или ключ API.".to_string());
        if lower.contains("openai") || lower.contains("sk-") {
            facts.push("Обнаружено упоминание API OpenAI / ChatGPT.".to_string());
        }
        if lower.contains("anthropic") || lower.contains("claude") {
            facts.push("Обнаружено упоминание API Anthropic / Claude.".to_string());
        }

        tests.push("Проверьте формат ключа (OpenAI: 'sk-proj-...', Anthropic: 'sk-ant-...').".to_string());
        tests.push("Выполните тестовый запрос curl с вашим ключом в заголовке Authorization.".to_string());

        fixes.push("Откройте настройки вашей IDE (Cursor, VS Code, Zed) и обновите сохраненный API-ключ.".to_string());
        fixes.push("Проверьте баланс аккаунта на портале разработчика (платформа может возвращать 401 при деактивации аккаунта).".to_string());
        fixes.push("Убедитесь, что переменная окружения OPENAI_API_KEY или ANTHROPIC_API_KEY не содержит лишних пробелов или кавычек.".to_string());

        return ErrorDetectiveResult {
            category: "AUTHENTICATION_ERROR".to_string(),
            error_code: "HTTP_401_UNAUTHORIZED".to_string(),
            title_ru: "Ошибка аутентификации API (HTTP 401)".to_string(),
            meaning_ru: "Сеть и интернет работают исправно, однако сервер ИИ отклонил ключ авторизации. Ключ не существует, отозван или не имеет прав.".to_string(),
            confirmed_facts: facts,
            likely_cause_ru: "Недействительный, истекший или неверно скопированный API-ключ в настройках IDE / проекта.".to_string(),
            suggested_tests: tests,
            safe_fix_ru: fixes,
            sanitized_input: sanitized,
        };
    }

    // 2. HTTP 403 Forbidden / Geo-blocking
    if (lower.contains("403") || lower.contains("forbidden") || lower.contains("451"))
        && (lower.contains("unsupported_country")
            || lower.contains("region")
            || lower.contains("geo")
            || lower.contains("country")
            || lower.contains("administrative rules")
            || lower.contains("cloudflare")
            || lower.contains("185.149.145.6")
            || lower.contains("access denied"))
    {
        facts.push("Сервер вернул HTTP 403 / 451: доступ запрещен политиками безопасности или географической фильтрации.".to_string());
        if lower.contains("185.149.145.6") || lower.contains("administrative rules") {
            facts.push("Обнаружен IP-адрес заглушки блокировки провайдера (185.149.145.6).".to_string());
        } else if lower.contains("cloudflare") {
            facts.push("Запрос был отклонен экраном защиты Cloudflare (WAF/Turnstile или блокировка по ASN/стране).".to_string());
        }

        tests.push("Запустите аудит DNS в AI Access Doctor, чтобы выявить подмену резолвера.".to_string());
        tests.push("Проверьте свой публичный IP и геопозицию через curl ifconfig.me.".to_string());

        fixes.push("Подключитесь через прокси или VPN с точкой выхода в поддерживаемой стране (США, Германия, Нидерланды).".to_string());
        fixes.push("Убедитесь, что прокси-клиент туннелирует не только веб-браузер, но и процесс IDE / Node.js.".to_string());

        return ErrorDetectiveResult {
            category: "GEO_OR_FIREWALL_BLOCK".to_string(),
            error_code: "HTTP_403_GEO_OR_FIREWALL".to_string(),
            title_ru: "Географическая блокировка или сетевой фильтр (HTTP 403)".to_string(),
            meaning_ru: "Сервер отказал в обслуживании из-за вашего географического региона либо запрос перехвачен шлюзом провайдера.".to_string(),
            confirmed_facts: facts,
            likely_cause_ru: "Сервисы OpenAI, Anthropic или Google Cloud блокируют запросы с IP-адресов неподдерживаемых регионов, либо провайдер применил фильтрацию.".to_string(),
            suggested_tests: tests,
            safe_fix_ru: fixes,
            sanitized_input: sanitized,
        };
    }

    // 3. HTTP 429 Too Many Requests / Quota Exceeded
    if lower.contains("429")
        || lower.contains("rate limit")
        || lower.contains("too many requests")
        || lower.contains("insufficient_quota")
        || lower.contains("quota exceeded")
    {
        facts.push("Сервер ответил HTTP 429: сетевой доступ идеален, но исчерпан лимит запросов в минуту (RPM/TPM) или баланс аккаунта.".to_string());
        if lower.contains("insufficient_quota") || lower.contains("exceeded your current quota") {
            facts.push("Причина отказа — исчерпание кредитного лимита аккаунта (Zero Balance), а не временный всплеск нагрузки.".to_string());
        }

        tests.push("Проверьте статус биллинга на платформе поставщика (OpenAI Billing / Anthropic Plans).".to_string());

        fixes.push("Пополните баланс API-аккаунта или добавьте платёжный метод.".to_string());
        fixes.push("Если это временный лимит (Rate Limit), настройте задержку между вызовами (exponential backoff) в плагине/коде.".to_string());

        return ErrorDetectiveResult {
            category: "QUOTA_OR_RATE_LIMIT".to_string(),
            error_code: "HTTP_429_RATE_LIMITED".to_string(),
            title_ru: "Исчерпан лимит запросов или баланс аккаунта (HTTP 429)".to_string(),
            meaning_ru: "Платформа ИИ временно или полностью приостановила обработку ваших обращений из-за ограничений тарифа или нулевого баланса.".to_string(),
            confirmed_facts: facts,
            likely_cause_ru: "Превышение квоты токенов в минуту либо отсутствие средств на счете разработчика.".to_string(),
            suggested_tests: tests,
            safe_fix_ru: fixes,
            sanitized_input: sanitized,
        };
    }

    // 4. HTTP 502 / 504 Bad Gateway / Proxy Egress Drop
    if lower.contains("502")
        || lower.contains("bad gateway")
        || lower.contains("504")
        || lower.contains("gateway timeout")
        || lower.contains("proxy returned 502")
    {
        facts.push("Промежуточный прокси-сервер или шлюз не смог связаться с целевым сервером назначения.".to_string());
        if lower.contains("127.0.0.1") || lower.contains("localhost") {
            facts.push("Сбой произошел на локальном хосте (127.0.0.1) в процессе локального прокси-клиента.".to_string());
        }

        tests.push("Проверьте раздел 'Сеть и Прокси' в AI Access Doctor для тестирования порта прокси.".to_string());
        tests.push("Выполните команду: Test-NetConnection -ComputerName 127.0.0.1 -Port <порт_прокси>".to_string());

        fixes.push("Убедитесь, что ваш прокси-клиент (v2ray, Xray, sing-box, Clash) запущен и не упал.".to_string());
        fixes.push("Проверьте совпадение порта в переменной HTTP_PROXY и в настройках работающего клиента.".to_string());
        fixes.push("Если прокси отключен, удалите переменные HTTP_PROXY и HTTPS_PROXY из среды Windows.".to_string());

        return ErrorDetectiveResult {
            category: "PROXY_GATEWAY_ERROR".to_string(),
            error_code: "HTTP_502_BAD_GATEWAY".to_string(),
            title_ru: "Ошибка промежуточного шлюза / прокси (HTTP 502 Bad Gateway)".to_string(),
            meaning_ru: "Локальный или сетевой прокси-клиент перехватил запрос, но не смог пробросить трафик в интернет.".to_string(),
            confirmed_facts: facts,
            likely_cause_ru: "Прокси-демон не запущен, завис, настроен на неверный порт, либо удаленный сервер прокси потерял связь.".to_string(),
            suggested_tests: tests,
            safe_fix_ru: fixes,
            sanitized_input: sanitized,
        };
    }

    // 5. Connection Reset / ECONNRESET
    if lower.contains("econnreset")
        || lower.contains("connection reset by peer")
        || lower.contains("wsaeconnreset")
        || lower.contains("connection was reset")
    {
        facts.push("Удаленная сторона или промежуточный шлюз внезапно разорвал установленное TCP-соединение (пакет TCP RST).".to_string());
        facts.push("Рукопожатие началось, но передача данных была прервана извне.".to_string());

        tests.push("Запустите проверку эндпоинтов AI Access Doctor, чтобы увидеть, на каком шаге происходит сброс.".to_string());

        fixes.push("Активируйте прокси-туннелирование (SOCKS5/HTTP) для обхода DPI-фильтров провайдера.".to_string());
        fixes.push("Временно отключите инспекцию сетевого трафика в антивирусе или брандмауэре.".to_string());

        return ErrorDetectiveResult {
            category: "NETWORK_TRANSPORT_RESET".to_string(),
            error_code: "SOCKET_ECONNRESET".to_string(),
            title_ru: "Принудительный сброс соединения (ECONNRESET)".to_string(),
            meaning_ru: "Сетевой пакет TCP RST принудительно оборвал сессию. Обычно это указывает на вмешательство DPI-фильтра провайдера или антивируса.".to_string(),
            confirmed_facts: facts,
            likely_cause_ru: "Блокировка трафика глубоким анализом пакетов (DPI) провайдера либо сетевым экраном.".to_string(),
            suggested_tests: tests,
            safe_fix_ru: fixes,
            sanitized_input: sanitized,
        };
    }

    // 6. Connection Refused / ECONNREFUSED
    if lower.contains("econnrefused")
        || lower.contains("connection refused")
        || lower.contains("wsaeconnrefused")
    {
        facts.push("Операционная система целевого хоста активно отклонила попытку подключения (порт закрыт).".to_string());
        if lower.contains("127.0.0.1") || lower.contains("localhost") {
            facts.push("Ошибка указывает на локальный адрес: программа пытается подключиться к несуществующему локальному сервису/порту.".to_string());
        }

        tests.push("Проверьте список слушающих портов в Windows: netstat -ano | findstr <порт>".to_string());

        fixes.push("Если в ошибке фигурирует локальный порт (например, 1080, 53200, 7890) — запустите соответствующий прокси-клиент.".to_string());
        fixes.push("Сбросьте устаревшие настройки прокси в переменных окружения Windows.".to_string());

        return ErrorDetectiveResult {
            category: "CONNECTION_REFUSED".to_string(),
            error_code: "SOCKET_ECONNREFUSED".to_string(),
            title_ru: "Соединение отклонено адресатом (ECONNREFUSED)".to_string(),
            meaning_ru: "На целевом IP-адресе и порту нет активной программы, готовой принять сетевое подключение.".to_string(),
            confirmed_facts: facts,
            likely_cause_ru: "Попытка обращения к закрытому локальному порту прокси или остановленной службе.".to_string(),
            suggested_tests: tests,
            safe_fix_ru: fixes,
            sanitized_input: sanitized,
        };
    }

    // 7. Timeout / ETIMEDOUT
    if lower.contains("etimedout")
        || lower.contains("connection timed out")
        || lower.contains("timed out")
        || lower.contains("request timeout")
    {
        facts.push("За отведенный период времени от удаленного узла не было получено ни одного ответного пакета (TCP SYN без ответа).".to_string());

        tests.push("Проверьте статус основного шлюза и пинг до 1.1.1.1 в AI Access Doctor.".to_string());

        fixes.push("Проверьте физическое подключение к сети и маршрутизатор.".to_string());
        fixes.push("Если тайм-аут возникает только при обращениях к сервисам ИИ — используйте прокси или VPN для обхода черных дыр (blackholing) провайдера.".to_string());

        return ErrorDetectiveResult {
            category: "TIMEOUT_ERROR".to_string(),
            error_code: "SOCKET_ETIMEDOUT".to_string(),
            title_ru: "Превышено время ожидания ответа (ETIMEDOUT)".to_string(),
            meaning_ru: "Пакеты уходят в сеть, но ответ не возвращается. Трафик бесследно сбрасывается маршрутизатором или провайдером.".to_string(),
            confirmed_facts: facts,
            likely_cause_ru: "Потеря пакетов, блокировка IP-подсети провайдером (blackholing) либо физический обрыв связи.".to_string(),
            suggested_tests: tests,
            safe_fix_ru: fixes,
            sanitized_input: sanitized,
        };
    }

    // 8. DNS / ENOTFOUND / EAI_AGAIN
    if lower.contains("enotfound")
        || lower.contains("getaddrinfo")
        || lower.contains("eai_again")
        || lower.contains("name or service not known")
        || lower.contains("dns resolution failed")
    {
        facts.push("Подсистема разрешения доменных имен (DNS) не смогла сопоставить текстовый домен с IP-адресом.".to_string());

        tests.push("Выполните команду в PowerShell: Resolve-DnsName <домен_из_ошибки>".to_string());
        tests.push("Сравните результат с независимым сервером: Resolve-DnsName <домен_из_ошибки> -Server 1.1.1.1".to_string());

        fixes.push("Очистите локальный кэш DNS: ipconfig /flushdns".to_string());
        fixes.push("Настройте в свойствах сетевого адаптера надежные публичные DNS: 1.1.1.1 и 8.8.8.8.".to_string());
        fixes.push("Включите DNS-over-HTTPS (DoH) для защиты от блокировок DNS провайдером.".to_string());

        return ErrorDetectiveResult {
            category: "DNS_ERROR".to_string(),
            error_code: "DNS_ENOTFOUND".to_string(),
            title_ru: "Ошибка разрешения доменного имени (ENOTFOUND / DNS)".to_string(),
            meaning_ru: "Компьютер не может найти IP-адрес сервера ИИ в телефонной книге интернета (DNS).".to_string(),
            confirmed_facts: facts,
            likely_cause_ru: "Сбой или фильтрация DNS-сервера вашего интернет-провайдера, либо повреждение кэша DNS.".to_string(),
            suggested_tests: tests,
            safe_fix_ru: fixes,
            sanitized_input: sanitized,
        };
    }

    // 9. TLS / SSL Certificate Validation Errors
    if lower.contains("certificate")
        || lower.contains("self-signed")
        || lower.contains("unable_to_verify_leaf_signature")
        || lower.contains("cert_has_expired")
        || lower.contains("depth_zero_self_signed_cert")
        || lower.contains("ssl_error_syscall")
        || lower.contains("sslcertverificationerror")
    {
        facts.push("Защищенное HTTPS-соединение было разорвано клиентом, так как представленный сертификат не признан доверенным.".to_string());
        facts.push("Корневой центр сертификации (Root CA) отсутствует в локальном хранилище доверенных сертификатов.".to_string());

        tests.push("Проверьте свойства сертификата через браузер или команду: openssl s_client -connect <host>:443".to_string());

        fixes.push("Если на компьютере установлен сторонний антивирус (Kaspersky, ESET, Bitdefender) — отключите в нем проверку защищенных соединений HTTPS.".to_string());
        fixes.push("Для приложений Node.js / Cursor задайте переменную NODE_EXTRA_CA_CERTS с путем к вашему корневому сертификату.".to_string());
        fixes.push("Для Git настройте: git config --global http.sslVerify true (или укажите http.sslCAInfo).".to_string());

        return ErrorDetectiveResult {
            category: "TLS_CERTIFICATE_ERROR".to_string(),
            error_code: "TLS_VERIFICATION_FAILED".to_string(),
            title_ru: "Ошибка проверки сертификата безопасности TLS / SSL".to_string(),
            meaning_ru: "Программа разработчика отклонила шифрованное соединение, заподозрив подмену сертификата или перехват трафика.".to_string(),
            confirmed_facts: facts,
            likely_cause_ru: "Перехват HTTPS-трафика локальным антивирусом, корпоративным прокси или повреждение цепочки доверенных CA.".to_string(),
            suggested_tests: tests,
            safe_fix_ru: fixes,
            sanitized_input: sanitized,
        };
    }

    // 10. Fallback / Generic
    facts.push("Введенный фрагмент проанализирован базой сигнатур сетевых сбоев.".to_string());
    tests.push("Запустите полную диагностику в AI Access Doctor для проверки всех слоев сетевого стека.".to_string());
    fixes.push("Проверьте статус подключения к интернету и перезапустите проблемное приложение.".to_string());

    ErrorDetectiveResult {
        category: "UNKNOWN_DIAGNOSTIC_PATTERN".to_string(),
        error_code: "GENERIC_NETWORK_ANOMALY".to_string(),
        title_ru: "Общая сетевая аномалия".to_string(),
        meaning_ru: "В тексте ошибки зафиксирован сетевой сбой, требующий комплексной проверки сетевого стека.".to_string(),
        confirmed_facts: facts,
        likely_cause_ru: "Нестандартное сообщение об ошибке среды выполнения или сторонней библиотеки.".to_string(),
        suggested_tests: tests,
        safe_fix_ru: fixes,
        sanitized_input: sanitized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_econnreset() {
        let log = "Error: read ECONNRESET at TLSWrap.onStreamRead (node:internal/stream_base_commons:217:20)";
        let res = analyze_error_text(log);
        assert_eq!(res.category, "NETWORK_TRANSPORT_RESET");
        assert_eq!(res.error_code, "SOCKET_ECONNRESET");
        assert!(res.meaning_ru.contains("TCP RST"));
    }

    #[test]
    fn test_detect_401_unauthorized() {
        let log = "openai.AuthenticationError: Error code: 401 - {'error': {'message': 'Incorrect API key provided: sk-proj-1234567890abcdef'}}";
        let res = analyze_error_text(log);
        assert_eq!(res.category, "AUTHENTICATION_ERROR");
        assert_eq!(res.error_code, "HTTP_401_UNAUTHORIZED");
        // Verify key was redacted
        assert!(!res.sanitized_input.contains("sk-proj-1234567890abcdef"));
        assert!(res.sanitized_input.contains("[REDACTED_KEY]"));
    }

    #[test]
    fn test_detect_502_bad_gateway() {
        let log = "FetchError: request to https://api.openai.com/v1/models failed, reason: HTTP 502 Bad Gateway via 127.0.0.1:53200";
        let res = analyze_error_text(log);
        assert_eq!(res.category, "PROXY_GATEWAY_ERROR");
        assert_eq!(res.error_code, "HTTP_502_BAD_GATEWAY");
    }

    #[test]
    fn test_detect_tls_leaf_signature() {
        let log = "FetchError: request failed, reason: unable to verify the first certificate (UNABLE_TO_VERIFY_LEAF_SIGNATURE)";
        let res = analyze_error_text(log);
        assert_eq!(res.category, "TLS_CERTIFICATE_ERROR");
        assert_eq!(res.error_code, "TLS_VERIFICATION_FAILED");
    }
}
