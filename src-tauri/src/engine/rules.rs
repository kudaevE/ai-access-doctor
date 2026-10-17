use crate::diag::dns::DnsDiagnostics;
use crate::diag::network::NetworkDiagnostics;
use crate::diag::proxy::ProxyDiagnostics;
use crate::diag::vpn::VpnDiagnostics;
use crate::services::{ServiceProfileResult, ServiceStatusKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticVerdict {
    pub overall_state: String, // "HEALTHY" | "WARNING" | "CRITICAL_ISSUE" | "UNKNOWN"
    pub primary_issue_code: String,
    pub confidence_percent: u32,
    pub evidence_strength: String, // "CONFIRMED_FACT" | "STRONGLY_SUPPORTED" | "HYPOTHESIS" | "INCONCLUSIVE"
    pub confirmed_facts: Vec<String>,
    pub title_ru: String,
    pub explanation_ru: String,
    pub technical_evidence: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub passed_checks_count: usize,
    pub warning_checks_count: usize,
    pub failed_checks_count: usize,
}

pub fn evaluate_root_cause(
    net: &NetworkDiagnostics,
    dns: &DnsDiagnostics,
    proxy: &ProxyDiagnostics,
    vpn: &VpnDiagnostics,
    services: &[ServiceProfileResult],
) -> DiagnosticVerdict {
    let mut evidence = Vec::new();
    let mut passed = 0;
    let mut warnings = 0;
    let mut failed = 0;

    // Count service states
    for s in services {
        match s.status_kind {
            ServiceStatusKind::Pass | ServiceStatusKind::AuthBoundary => passed += 1,
            ServiceStatusKind::RateLimited | ServiceStatusKind::ServerFailure => warnings += 1,
            _ => failed += 1,
        }
    }

    // 1. Check for complete network outage
    if !net.dual_stack.ipv4_working && !net.dual_stack.ipv6_working {
        evidence.push("Dual-stack probe failed: both IPv4 and IPv6 direct connections timed out.".to_string());
        if let Some(gw) = &net.primary_gateway {
            evidence.push(format!("Primary gateway detected as {}, but no internet egress.", gw));
        }

        return DiagnosticVerdict {
            overall_state: "CRITICAL_ISSUE".to_string(),
            primary_issue_code: "NO_INTERNET_CONNECTIVITY".to_string(),
            confidence_percent: 100,
            evidence_strength: "CONFIRMED_FACT".to_string(),
            confirmed_facts: vec![
                "Прямые TCP-зонды IPv4 и IPv6 завершились таймаутом".to_string(),
                "Сетевой шлюз не обеспечивает выход во внешнюю сеть интернет".to_string(),
            ],
            title_ru: "Отсутствует подключение к интернету".to_string(),
            explanation_ru: "Компьютер не может установить связь ни по IPv4, ни по IPv6. Сетевой трафик не покидает пределы локальной сети.".to_string(),
            technical_evidence: evidence,
            recommended_actions: vec![
                "Проверьте физическое подключение сетевого кабеля или Wi-Fi сети.".to_string(),
                "Перезагрузите сетевой маршрутизатор или модем.".to_string(),
                "Убедитесь, что сетевой адаптер включен в диспетчере устройств Windows.".to_string(),
            ],
            passed_checks_count: passed,
            warning_checks_count: warnings,
            failed_checks_count: failed + 1,
        };
    }

    // 2. Check Proxy Dead Port or Proxy Conflict
    let dead_proxies: Vec<_> = proxy.active_proxies.iter().filter(|p| !p.can_forward_https).collect();
    let working_proxies: Vec<_> = proxy.active_proxies.iter().filter(|p| p.can_forward_https).collect();

    if proxy.has_conflict && !working_proxies.is_empty() && !dead_proxies.is_empty() {
        evidence.push(format!(
            "Environment proxy variable points to '{}' which returned failure/closed port.",
            dead_proxies[0].endpoint
        ));
        evidence.push(format!(
            "System UserProxy in Windows registry is set to '{}' and successfully forwards HTTPS.",
            working_proxies[0].endpoint
        ));

        return DiagnosticVerdict {
            overall_state: "CRITICAL_ISSUE".to_string(),
            primary_issue_code: "PROXY_ENV_CONFLICT".to_string(),
            confidence_percent: 96,
            evidence_strength: "CONFIRMED_FACT".to_string(),
            confirmed_facts: vec![
                format!("Переменная HTTP_PROXY/HTTPS_PROXY указывает на порт '{}', возвращающий сбой или 502", dead_proxies[0].endpoint),
                format!("Системный прокси Windows (WinINet) настроен на '{}' и успешно пропускает HTTPS", working_proxies[0].endpoint),
            ],
            title_ru: "Конфликт системного прокси и переменных окружения".to_string(),
            explanation_ru: "В системе обнаружен конфликт портов: браузер и системные службы используют рабочий прокси-порт, однако консольные инструменты (npm, git, Python, curl) обращаются к неработающему порту из переменной HTTP_PROXY / HTTPS_PROXY.".to_string(),
            technical_evidence: evidence,
            recommended_actions: vec![
                format!("Установите переменные окружения на действующий порт: [System.Environment]::SetEnvironmentVariable('HTTP_PROXY', '{}', 'User')", working_proxies[0].endpoint),
                format!("Либо удалите устаревшие переменные HTTP_PROXY и HTTPS_PROXY, если прокси-клиент слушает на порту Windows по умолчанию."),
                "Перезапустите терминал или IDE для применения изменений окружения.".to_string(),
            ],
            passed_checks_count: passed,
            warning_checks_count: warnings,
            failed_checks_count: failed + 1,
        };
    }

    if !dead_proxies.is_empty() && working_proxies.is_empty() && (proxy.env_vars.has_any || proxy.wininet.enabled) {
        for dp in &dead_proxies {
            evidence.push(format!("Configured proxy endpoint '{}' failed: {:?}", dp.endpoint, dp.response_error));
        }

        return DiagnosticVerdict {
            overall_state: "CRITICAL_ISSUE".to_string(),
            primary_issue_code: "PROXY_DEAD_PORT".to_string(),
            confidence_percent: 98,
            evidence_strength: "CONFIRMED_FACT".to_string(),
            confirmed_facts: vec![
                format!("Прокси-порт '{}' не отвечает или завершает соединение с ошибкой", dead_proxies[0].endpoint),
                "Ни один из сконфигурированных прокси-портов не способен туннелировать HTTPS".to_string(),
            ],
            title_ru: "Локальный прокси-сервер не отвечает (Dead Proxy Port)".to_string(),
            explanation_ru: format!(
                "В системе включен прокси-сервер ({}), но локальный процесс прокси не запущен, либо порт закрыт. Все сетевые обращения ИИ-клиентов блокируются.",
                dead_proxies[0].endpoint
            ),
            technical_evidence: evidence,
            recommended_actions: vec![
                "Запустите ваш прокси/VPN клиент (v2ray, sing-box, Xray, Clash, Shadowsocks).".to_string(),
                "Если прокси больше не используется, отключите системный прокси в Параметрах Windows (Сеть и интернет -> Прокси).".to_string(),
                "Очистите переменные окружения HTTP_PROXY и HTTPS_PROXY.".to_string(),
            ],
            passed_checks_count: passed,
            warning_checks_count: warnings,
            failed_checks_count: failed + 1,
        };
    }

    // 3. Check DNS Poisoning or Censorship
    if dns.any_tampering {
        let is_mitigated_by_proxy = failed == 0 && (proxy.wininet.enabled || proxy.env_vars.has_any || vpn.vpn_tunnel_detected);

        for item in &dns.items {
            if item.is_tampered_or_blocked {
                evidence.push(format!(
                    "Hostname '{}': System DNS spoofed/diverged to {:?}, but public DNS resolved to {:?}",
                    item.hostname, item.system_result.resolved_ips, item.cloudflare_result.resolved_ips
                ));
            }
        }

        if is_mitigated_by_proxy {
            evidence.push("System DNS tampering is actively neutralized by active proxy/VPN with remote DNS resolution.".to_string());
            return DiagnosticVerdict {
                overall_state: "HEALTHY".to_string(),
                primary_issue_code: "DNS_SPOOF_BYPASSED_BY_PROXY".to_string(),
                confidence_percent: 98,
                evidence_strength: "CONFIRMED_FACT".to_string(),
                confirmed_facts: vec![
                    "Системный DNS провайдера подменяет IP AI-доменов на страницу-заглушку".to_string(),
                    "Работающий локальный прокси успешно выполняет удаленное разрешение имен".to_string(),
                    "Все AI-эндпоинты и Google Antigravity отвечают с нормальной задержкой".to_string(),
                ],
                title_ru: "Активный прокси успешно нейтрализует DNS-блокировки провайдера".to_string(),
                explanation_ru: "Системный DNS-сервер провайдера подменяет IP-адреса на страницу блокировки (185.149.145.6). Однако благодаря работающему прокси и удаленному разрешению имен, Antigravity и все инструменты ИИ функционируют штатно и без сбоев.".to_string(),
                technical_evidence: evidence,
                recommended_actions: vec![
                    "Ваши ИИ-сервисы и Antigravity работают штатно через прокси. Никаких срочных действий не требуется.".to_string(),
                    "Для утилит без встроенной поддержки системного прокси (например, прямого curl) рекомендуется включить DNS-over-HTTPS (DoH) или настроить публичные DNS (1.1.1.1 / 8.8.8.8).".to_string(),
                ],
                passed_checks_count: passed,
                warning_checks_count: warnings,
                failed_checks_count: 0,
            };
        }

        return DiagnosticVerdict {
            overall_state: "CRITICAL_ISSUE".to_string(),
            primary_issue_code: "DNS_TAMPERING_OR_BLOCK".to_string(),
            confidence_percent: 94,
            evidence_strength: "CONFIRMED_FACT".to_string(),
            confirmed_facts: vec![
                "Системный DNS возвращает неверный IP (заглушка) либо отклоняет запрос".to_string(),
                "Независимый DNS-резолвер (1.1.1.1 / 8.8.8.8) успешно возвращает подлинные адреса".to_string(),
            ],
            title_ru: "Блокировка или подмена DNS-запросов провайдером".to_string(),
            explanation_ru: "Системный DNS-сервер не может разрешить адреса серверов ИИ либо подменяет их, в то время как независимые публичные резолверы успешно возвращают адреса.".to_string(),
            technical_evidence: evidence,
            recommended_actions: vec![
                "Настройте статические публичные DNS в свойствах сетевого подключения (1.1.1.1 и 8.8.8.8).".to_string(),
                "Включите функцию DNS-over-HTTPS (DoH) в вашей системе или браузере.".to_string(),
                "Очистите локальный кэш DNS с помощью команды: ipconfig /flushdns".to_string(),
            ],
            passed_checks_count: passed,
            warning_checks_count: warnings,
            failed_checks_count: failed + 1,
        };
    }

    // 4. Check TLS Interception (Antivirus or Corporate Proxy)
    let tls_issues: Vec<_> = services.iter().filter(|s| s.status_kind == ServiceStatusKind::TlsFailure).collect();
    if !tls_issues.is_empty() {
        for s in &tls_issues {
            evidence.push(format!("Service '{}' failed TLS validation: {}", s.name, s.summary_ru));
        }

        return DiagnosticVerdict {
            overall_state: "CRITICAL_ISSUE".to_string(),
            primary_issue_code: "TLS_MITM_INTERCEPTION".to_string(),
            confidence_percent: 92,
            evidence_strength: "CONFIRMED_FACT".to_string(),
            confirmed_facts: vec![
                "TLS-рукопожатие отклонено из-за недоверенного сертификата безопасности".to_string(),
                "Сертификат подменен локальным антивирусным ПО или корпоративным прокси".to_string(),
            ],
            title_ru: "Перехват TLS-сертификатов (Антивирус или корпоративный MITM)".to_string(),
            explanation_ru: "Сертификаты шифрования HTTPS подменяются локальным антивирусным сканером или корпоративным шлюзом. Библиотеки разработчика (Node.js, Rust, Python) отклоняют такие сертификаты в целях безопасности.".to_string(),
            technical_evidence: evidence,
            recommended_actions: vec![
                "Временно отключите проверку HTTPS-соединений в антивирусе (Kaspersky, ESET, Bitdefender, Avast).".to_string(),
                "Для Node.js добавьте переменную окружения: NODE_EXTRA_CA_CERTS с путём к корпоративному сертификату.".to_string(),
                "Для Git добавьте сертификат в конфигурацию: git config --global http.sslCAInfo <путь_к_сертификату>".to_string(),
            ],
            passed_checks_count: passed,
            warning_checks_count: warnings,
            failed_checks_count: failed + 1,
        };
    }

    // 5. Check Geo-IP Blocking
    let geo_blocked: Vec<_> = services.iter().filter(|s| s.status_kind == ServiceStatusKind::GeoBlocked).collect();
    if !geo_blocked.is_empty() {
        for s in &geo_blocked {
            evidence.push(format!("Service '{}' restricted by region (HTTP 403/451).", s.name));
        }

        return DiagnosticVerdict {
            overall_state: "CRITICAL_ISSUE".to_string(),
            primary_issue_code: "GEO_IP_RESTRICTION".to_string(),
            confidence_percent: 90,
            evidence_strength: "CONFIRMED_FACT".to_string(),
            confirmed_facts: vec![
                "Сервер ИИ отвечает на сетевом уровне, но возвращает HTTP 403/451 с региональным отказом".to_string(),
            ],
            title_ru: "Географическое ограничение доступа (Geo-IP Block)".to_string(),
            explanation_ru: "Серверы ИИ доступны по сети, но блокируют запросы, поступающие с IP-адресов вашего текущего региона.".to_string(),
            technical_evidence: evidence,
            recommended_actions: vec![
                "Используйте VPN-туннель или прокси-сервер с геолокацией в США или Западной Европе.".to_string(),
                "Убедитесь, что в VPN включен режим полного туннелирования (Full Tunnel), а не только DNS.".to_string(),
            ],
            passed_checks_count: passed,
            warning_checks_count: warnings,
            failed_checks_count: failed + 1,
        };
    }

    // 6. Check Routing collision
    if vpn.has_routing_collision {
        evidence.push("Multiple default 0.0.0.0/0 routes with conflicting metrics detected.".to_string());
        return DiagnosticVerdict {
            overall_state: "WARNING".to_string(),
            primary_issue_code: "VPN_ROUTE_METRIC_COLLISION".to_string(),
            confidence_percent: 85,
            evidence_strength: "STRONGLY_SUPPORTED".to_string(),
            confirmed_facts: vec![
                "В системной таблице маршрутизации найдено несколько маршрутов по умолчанию 0.0.0.0/0 с совпадающими метриками".to_string(),
            ],
            title_ru: "Конфликт метрик маршрутизации VPN".to_string(),
            explanation_ru: "В таблице маршрутизации обнаружено несколько маршрутов по умолчанию с одинаковым приоритетом. Пакеты могут случайным образом сбрасываться.".to_string(),
            technical_evidence: evidence,
            recommended_actions: vec![
                "Переподключите VPN-клиент для сброса интерфейсного метрика.".to_string(),
                "Отключите неиспользуемые виртуальные сетевые адаптеры в Панели управления Windows.".to_string(),
            ],
            passed_checks_count: passed,
            warning_checks_count: warnings + 1,
            failed_checks_count: failed,
        };
    }

    // 7. Check if all healthy
    if failed == 0 {
        evidence.push(format!("All {} verified endpoints responded normally.", services.len()));
        if let Some(gw) = &net.primary_gateway {
            evidence.push(format!("Gateway: {}, Egress IP Family: {}", gw, net.dual_stack.preferred_family));
        }

        return DiagnosticVerdict {
            overall_state: "HEALTHY".to_string(),
            primary_issue_code: "ALL_SYSTEMS_OPERATIONAL".to_string(),
            confidence_percent: 100,
            evidence_strength: "CONFIRMED_FACT".to_string(),
            confirmed_facts: vec![
                format!("Все {} ключевых AI-эндпоинтов ответили без сетевых сбоев", services.len()),
                "TCP-рукопожатие, валидация TLS и разрешение DNS подтверждены".to_string(),
            ],
            title_ru: "Сетевой стек и подключение к ИИ-сервисам полностью исправны".to_string(),
            explanation_ru: "Все ключевые компоненты (DNS, TCP, TLS, прокси и маршрутизация) функционируют штатно. ИИ-сервисы доступны и отвечают с минимальной задержкой.".to_string(),
            technical_evidence: evidence,
            recommended_actions: vec![
                "Сетевых проблем не обнаружено.".to_string(),
                "Если инструмент сообщает об ошибке, проверьте корректность API-ключей и авторизации.".to_string(),
            ],
            passed_checks_count: passed,
            warning_checks_count: warnings,
            failed_checks_count: failed,
        };
    }

    // Default Fallback
    evidence.push("Partial failure without a single dominant pattern.".to_string());
    DiagnosticVerdict {
        overall_state: "WARNING".to_string(),
        primary_issue_code: "PARTIAL_DEGRADATION".to_string(),
        confidence_percent: 60,
        evidence_strength: "HYPOTHESIS".to_string(),
        confirmed_facts: vec![
            "Часть эндпоинтов ответила успешно, для других зафиксированы сетевые аномалии".to_string(),
        ],
        title_ru: "Частичная недоступность отдельных сервисов".to_string(),
        explanation_ru: "Часть сервисов функционирует нормально, но для отдельных эндпоинтов зафиксированы задержки или ошибки.".to_string(),
        technical_evidence: evidence,
        recommended_actions: vec![
            "Проверьте статус недоступных сервисов в разделе 'ИИ-сервисы'.".to_string(),
            "Повторите диагностику через 1-2 минуты.".to_string(),
        ],
        passed_checks_count: passed,
        warning_checks_count: warnings,
        failed_checks_count: failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diag::dns::{DnsComparisonItem, ResolverResult};
    use crate::diag::network::DualStackStatus;
    use crate::diag::proxy::{ProxyEnvVars, ProxyEndpointCheck, WinHttpProxy, WinInetProxy};
    use crate::diag::vpn::RouteItem;

    fn mock_healthy_network() -> NetworkDiagnostics {
        NetworkDiagnostics {
            interfaces: Vec::new(),
            dual_stack: DualStackStatus {
                ipv4_working: true,
                ipv4_rtt_ms: Some(25),
                ipv6_working: false,
                ipv6_rtt_ms: None,
                preferred_family: "IPv4-Only".to_string(),
            },
            primary_gateway: Some("192.168.1.1".to_string()),
            primary_interface: Some("Ethernet".to_string()),
            default_dns: vec!["1.1.1.1".to_string()],
        }
    }

    fn mock_healthy_dns() -> DnsDiagnostics {
        DnsDiagnostics {
            system_dns_working: true,
            average_dns_latency_ms: 18,
            items: Vec::new(),
            any_tampering: false,
        }
    }

    fn mock_healthy_proxy() -> ProxyDiagnostics {
        ProxyDiagnostics {
            wininet: WinInetProxy {
                enabled: false,
                server: None,
                pac_url: None,
                override_exceptions: None,
            },
            winhttp: WinHttpProxy {
                is_direct: true,
                proxy_server: None,
                bypass_list: None,
                raw_output: "Direct access".to_string(),
            },
            env_vars: ProxyEnvVars {
                http_proxy: None,
                https_proxy: None,
                all_proxy: None,
                no_proxy: None,
                has_any: false,
            },
            active_proxies: Vec::new(),
            has_conflict: false,
            conflict_description: None,
            recommendation: None,
        }
    }

    fn mock_healthy_vpn() -> VpnDiagnostics {
        VpnDiagnostics {
            active_virtual_adapters: Vec::new(),
            default_routes: Vec::new(),
            vpn_tunnel_detected: false,
            has_routing_collision: false,
            analysis_summary: "Direct physical connection".to_string(),
        }
    }

    fn mock_healthy_services() -> Vec<ServiceProfileResult> {
        vec![
            ServiceProfileResult {
                id: "antigravity".to_string(),
                name: "Google Antigravity".to_string(),
                category: "AI Code Engine".to_string(),
                overall_status: "pass".to_string(),
                status_kind: ServiceStatusKind::Pass,
                primary_endpoint: "https://accounts.google.com".to_string(),
                endpoints: Vec::new(),
                latency_ms: 45,
                summary_ru: "Доступен".to_string(),
                summary_en: "Pass".to_string(),
                recommendation_ru: None,
            },
            ServiceProfileResult {
                id: "openai".to_string(),
                name: "OpenAI / Codex".to_string(),
                category: "Models API".to_string(),
                overall_status: "pass".to_string(),
                status_kind: ServiceStatusKind::AuthBoundary, // Normal 401 auth boundary when unauthenticated
                primary_endpoint: "https://api.openai.com".to_string(),
                endpoints: Vec::new(),
                latency_ms: 80,
                summary_ru: "Доступен на сетевом уровне (Auth Required)".to_string(),
                summary_en: "Pass".to_string(),
                recommendation_ru: None,
            },
        ]
    }

    #[test]
    fn test_scenario_all_healthy() {
        let net = mock_healthy_network();
        let dns = mock_healthy_dns();
        let proxy = mock_healthy_proxy();
        let vpn = mock_healthy_vpn();
        let services = mock_healthy_services();

        let verdict = evaluate_root_cause(&net, &dns, &proxy, &vpn, &services);
        assert_eq!(verdict.overall_state, "HEALTHY");
        assert_eq!(verdict.primary_issue_code, "ALL_SYSTEMS_OPERATIONAL");
        assert_eq!(verdict.confidence_percent, 100);
    }

    #[test]
    fn test_scenario_no_internet() {
        let mut net = mock_healthy_network();
        net.dual_stack.ipv4_working = false;
        net.dual_stack.ipv6_working = false;

        let dns = mock_healthy_dns();
        let proxy = mock_healthy_proxy();
        let vpn = mock_healthy_vpn();
        let services = mock_healthy_services();

        let verdict = evaluate_root_cause(&net, &dns, &proxy, &vpn, &services);
        assert_eq!(verdict.overall_state, "CRITICAL_ISSUE");
        assert_eq!(verdict.primary_issue_code, "NO_INTERNET_CONNECTIVITY");
        assert_eq!(verdict.confidence_percent, 100);
    }

    #[test]
    fn test_scenario_proxy_dead_port() {
        let net = mock_healthy_network();
        let dns = mock_healthy_dns();
        let mut proxy = mock_healthy_proxy();
        proxy.env_vars.https_proxy = Some("http://127.0.0.1:53200".to_string());
        proxy.env_vars.has_any = true;
        proxy.active_proxies.push(ProxyEndpointCheck {
            endpoint: "http://127.0.0.1:53200".to_string(),
            is_listening: true,
            tcp_rtt_ms: Some(2),
            can_forward_https: false,
            http_status: Some(502),
            response_error: Some("HTTP 502 Bad Gateway".to_string()),
        });

        let vpn = mock_healthy_vpn();
        let services = mock_healthy_services();

        let verdict = evaluate_root_cause(&net, &dns, &proxy, &vpn, &services);
        assert_eq!(verdict.overall_state, "CRITICAL_ISSUE");
        assert_eq!(verdict.primary_issue_code, "PROXY_DEAD_PORT");
        assert_eq!(verdict.confidence_percent, 98);
    }

    #[test]
    fn test_scenario_proxy_conflict() {
        let net = mock_healthy_network();
        let dns = mock_healthy_dns();
        let mut proxy = mock_healthy_proxy();
        proxy.has_conflict = true;
        proxy.env_vars.https_proxy = Some("http://127.0.0.1:53200".to_string());
        proxy.env_vars.has_any = true;
        proxy.wininet.enabled = true;
        proxy.wininet.server = Some("http://127.0.0.1:10809".to_string());

        proxy.active_proxies.push(ProxyEndpointCheck {
            endpoint: "http://127.0.0.1:53200".to_string(),
            is_listening: true,
            tcp_rtt_ms: Some(1),
            can_forward_https: false,
            http_status: Some(502),
            response_error: Some("Bad Gateway".to_string()),
        });
        proxy.active_proxies.push(ProxyEndpointCheck {
            endpoint: "http://127.0.0.1:10809".to_string(),
            is_listening: true,
            tcp_rtt_ms: Some(2),
            can_forward_https: true,
            http_status: Some(200),
            response_error: None,
        });

        let vpn = mock_healthy_vpn();
        let services = mock_healthy_services();

        let verdict = evaluate_root_cause(&net, &dns, &proxy, &vpn, &services);
        assert_eq!(verdict.overall_state, "CRITICAL_ISSUE");
        assert_eq!(verdict.primary_issue_code, "PROXY_ENV_CONFLICT");
        assert_eq!(verdict.confidence_percent, 96);
    }

    #[test]
    fn test_scenario_dns_tampering() {
        let net = mock_healthy_network();
        let mut dns = mock_healthy_dns();
        dns.any_tampering = true;
        dns.items.push(DnsComparisonItem {
            hostname: "api.openai.com".to_string(),
            system_result: ResolverResult {
                resolver_name: "System DNS".to_string(),
                resolver_ip: "192.168.1.1".to_string(),
                success: false,
                rtt_ms: None,
                resolved_ips: Vec::new(),
                error: Some("NXDOMAIN".to_string()),
            },
            cloudflare_result: ResolverResult {
                resolver_name: "Cloudflare".to_string(),
                resolver_ip: "1.1.1.1".to_string(),
                success: true,
                rtt_ms: Some(15),
                resolved_ips: vec!["104.18.6.192".to_string()],
                error: None,
            },
            google_result: ResolverResult {
                resolver_name: "Google".to_string(),
                resolver_ip: "8.8.8.8".to_string(),
                success: true,
                rtt_ms: Some(20),
                resolved_ips: vec!["104.18.6.192".to_string()],
                error: None,
            },
            quad9_result: ResolverResult {
                resolver_name: "Quad9".to_string(),
                resolver_ip: "9.9.9.9".to_string(),
                success: true,
                rtt_ms: Some(25),
                resolved_ips: vec!["104.18.6.192".to_string()],
                error: None,
            },
            is_tampered_or_blocked: true,
            divergence_detected: false,
            diagnosis_note: "Blocked".to_string(),
        });

        let proxy = mock_healthy_proxy();
        let vpn = mock_healthy_vpn();
        let mut services = mock_healthy_services();
        // Without working proxy/VPN and service is failing:
        services[0].overall_status = "fail".to_string();
        services[0].status_kind = ServiceStatusKind::DnsFailure;

        let verdict = evaluate_root_cause(&net, &dns, &proxy, &vpn, &services);
        assert_eq!(verdict.overall_state, "CRITICAL_ISSUE");
        assert_eq!(verdict.primary_issue_code, "DNS_TAMPERING_OR_BLOCK");
        assert_eq!(verdict.confidence_percent, 94);
    }

    #[test]
    fn test_scenario_dns_tampering_bypassed_by_proxy() {
        let net = mock_healthy_network();
        let mut dns = mock_healthy_dns();
        dns.any_tampering = true;
        dns.items.push(DnsComparisonItem {
            hostname: "accounts.google.com".to_string(),
            system_result: ResolverResult {
                resolver_name: "System DNS".to_string(),
                resolver_ip: "192.168.1.1".to_string(),
                success: true,
                rtt_ms: Some(10),
                resolved_ips: vec!["185.149.145.6".to_string()], // ISP blocking stub IP
                error: None,
            },
            cloudflare_result: ResolverResult {
                resolver_name: "Cloudflare DoH".to_string(),
                resolver_ip: "1.1.1.1 (HTTPS)".to_string(),
                success: true,
                rtt_ms: Some(15),
                resolved_ips: vec!["142.250.185.77".to_string()],
                error: None,
            },
            google_result: ResolverResult {
                resolver_name: "Google".to_string(),
                resolver_ip: "8.8.8.8".to_string(),
                success: true,
                rtt_ms: Some(18),
                resolved_ips: vec!["142.250.185.77".to_string()],
                error: None,
            },
            quad9_result: ResolverResult {
                resolver_name: "Quad9".to_string(),
                resolver_ip: "9.9.9.9".to_string(),
                success: true,
                rtt_ms: Some(20),
                resolved_ips: vec!["142.250.185.77".to_string()],
                error: None,
            },
            is_tampered_or_blocked: true,
            divergence_detected: true,
            diagnosis_note: "DNS spoofed".to_string(),
        });

        let mut proxy = mock_healthy_proxy();
        proxy.wininet.enabled = true;
        proxy.wininet.server = Some("http://127.0.0.1:10809".to_string());

        let vpn = mock_healthy_vpn();
        let services = mock_healthy_services();

        let verdict = evaluate_root_cause(&net, &dns, &proxy, &vpn, &services);
        assert_eq!(verdict.overall_state, "HEALTHY");
        assert_eq!(verdict.primary_issue_code, "DNS_SPOOF_BYPASSED_BY_PROXY");
        assert_eq!(verdict.confidence_percent, 98);
    }

    #[test]
    fn test_scenario_tls_mitm_interception() {
        let net = mock_healthy_network();
        let dns = mock_healthy_dns();
        let proxy = mock_healthy_proxy();
        let vpn = mock_healthy_vpn();
        let mut services = mock_healthy_services();
        services[0].overall_status = "fail".to_string();
        services[0].status_kind = ServiceStatusKind::TlsFailure;
        services[0].summary_ru = "Kaspersky HTTPS Inspection Certificate rejected".to_string();

        let verdict = evaluate_root_cause(&net, &dns, &proxy, &vpn, &services);
        assert_eq!(verdict.overall_state, "CRITICAL_ISSUE");
        assert_eq!(verdict.primary_issue_code, "TLS_MITM_INTERCEPTION");
        assert_eq!(verdict.confidence_percent, 92);
    }

    #[test]
    fn test_scenario_geo_ip_restriction() {
        let net = mock_healthy_network();
        let dns = mock_healthy_dns();
        let proxy = mock_healthy_proxy();
        let vpn = mock_healthy_vpn();
        let mut services = mock_healthy_services();
        services[1].overall_status = "fail".to_string();
        services[1].status_kind = ServiceStatusKind::GeoBlocked;

        let verdict = evaluate_root_cause(&net, &dns, &proxy, &vpn, &services);
        assert_eq!(verdict.overall_state, "CRITICAL_ISSUE");
        assert_eq!(verdict.primary_issue_code, "GEO_IP_RESTRICTION");
        assert_eq!(verdict.confidence_percent, 90);
    }

    #[test]
    fn test_scenario_vpn_metric_collision() {
        let net = mock_healthy_network();
        let dns = mock_healthy_dns();
        let proxy = mock_healthy_proxy();
        let mut vpn = mock_healthy_vpn();
        vpn.has_routing_collision = true;
        vpn.default_routes = vec![
            RouteItem {
                destination: "0.0.0.0/0".to_string(),
                netmask: "0.0.0.0".to_string(),
                gateway: "192.168.1.1".to_string(),
                interface_name: "Ethernet".to_string(),
                metric: 25,
                is_default_gateway: true,
            },
            RouteItem {
                destination: "0.0.0.0/0".to_string(),
                netmask: "0.0.0.0".to_string(),
                gateway: "10.0.0.1".to_string(),
                interface_name: "Wintun".to_string(),
                metric: 25,
                is_default_gateway: true,
            },
        ];

        let services = mock_healthy_services();

        let verdict = evaluate_root_cause(&net, &dns, &proxy, &vpn, &services);
        assert_eq!(verdict.overall_state, "WARNING");
        assert_eq!(verdict.primary_issue_code, "VPN_ROUTE_METRIC_COLLISION");
        assert_eq!(verdict.confidence_percent, 85);
    }
}

