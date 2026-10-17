use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolverResult {
    pub resolver_name: String,
    pub resolver_ip: String,
    pub success: bool,
    pub rtt_ms: Option<u64>,
    pub resolved_ips: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsComparisonItem {
    pub hostname: String,
    pub system_result: ResolverResult,
    pub cloudflare_result: ResolverResult,
    pub google_result: ResolverResult,
    pub quad9_result: ResolverResult,
    pub is_tampered_or_blocked: bool,
    pub divergence_detected: bool,
    pub diagnosis_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsDiagnostics {
    pub system_dns_working: bool,
    pub average_dns_latency_ms: u64,
    pub items: Vec<DnsComparisonItem>,
    pub any_tampering: bool,
}

/// Resolve a hostname using standard OS system resolver
pub async fn resolve_system_dns(hostname: &str) -> ResolverResult {
    let start = Instant::now();

    match tokio::net::lookup_host((hostname, 443u16)).await {
        Ok(addrs) => {
            let rtt = start.elapsed().as_millis() as u64;
            let ips: Vec<String> = addrs.map(|a| a.ip().to_string()).collect();
            let success = !ips.is_empty();
            ResolverResult {
                resolver_name: "System DNS".to_string(),
                resolver_ip: "Local Host / OS".to_string(),
                success,
                rtt_ms: Some(rtt),
                resolved_ips: ips,
                error: if success { None } else { Some("Empty address list".to_string()) },
            }
        }
        Err(e) => ResolverResult {
            resolver_name: "System DNS".to_string(),
            resolver_ip: "Local Host / OS".to_string(),
            success: false,
            rtt_ms: None,
            resolved_ips: Vec::new(),
            error: Some(e.to_string()),
        },
    }
}

/// Construct raw DNS query packet (A record, IN class)
fn build_dns_query(hostname: &str, transaction_id: u16) -> Vec<u8> {
    let mut packet = Vec::with_capacity(512);

    // Transaction ID (2 bytes)
    packet.extend_from_slice(&transaction_id.to_be_bytes());

    // Flags: Standard query, recursion desired (0x0100)
    packet.extend_from_slice(&[0x01, 0x00]);

    // Questions count: 1
    packet.extend_from_slice(&[0x00, 0x01]);

    // Answer RRs: 0, Authority RRs: 0, Additional RRs: 0
    packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    // QNAME
    for label in hostname.split('.') {
        let len = label.len() as u8;
        packet.push(len);
        packet.extend_from_slice(label.as_bytes());
    }
    packet.push(0x00); // End of QNAME

    // QTYPE: A (0x0001)
    packet.extend_from_slice(&[0x00, 0x01]);

    // QCLASS: IN (0x0001)
    packet.extend_from_slice(&[0x00, 0x01]);

    packet
}

/// Parse basic IPv4 addresses from DNS response packet
fn parse_dns_response(buf: &[u8]) -> Result<Vec<String>, String> {
    if buf.len() < 12 {
        return Err("DNS response too short".to_string());
    }

    let flags = u16::from_be_bytes([buf[2], buf[3]]);
    let rcode = flags & 0x000F;
    if rcode != 0 {
        return match rcode {
            3 => Err("NXDOMAIN (Non-Existent Domain)".to_string()),
            2 => Err("SERVFAIL (Server Failure)".to_string()),
            5 => Err("REFUSED (Query Refused)".to_string()),
            _ => Err(format!("DNS Error RCODE: {}", rcode)),
        };
    }

    let ancount = u16::from_be_bytes([buf[6], buf[7]]);
    if ancount == 0 {
        return Ok(Vec::new());
    }

    // Skip question section
    let mut offset = 12;
    while offset < buf.len() {
        let len = buf[offset] as usize;
        if len == 0 {
            offset += 1; // End of QNAME
            break;
        }
        if (len & 0xC0) == 0xC0 {
            offset += 2; // Pointer
            break;
        }
        offset += 1 + len;
    }
    offset += 4; // Skip QTYPE (2) + QCLASS (2)

    let mut ips = Vec::new();
    for _ in 0..ancount {
        if offset + 12 > buf.len() {
            break;
        }

        // Handle NAME (pointer or labels)
        if (buf[offset] & 0xC0) == 0xC0 {
            offset += 2;
        } else {
            while offset < buf.len() && buf[offset] != 0 {
                offset += 1 + (buf[offset] as usize);
            }
            offset += 1;
        }

        if offset + 10 > buf.len() {
            break;
        }

        let rtype = u16::from_be_bytes([buf[offset], buf[offset + 1]]);
        let _rclass = u16::from_be_bytes([buf[offset + 2], buf[offset + 3]]);
        let _ttl = u32::from_be_bytes([buf[offset + 4], buf[offset + 5], buf[offset + 6], buf[offset + 7]]);
        let rdlength = u16::from_be_bytes([buf[offset + 8], buf[offset + 9]]) as usize;
        offset += 10;

        if rtype == 1 && rdlength == 4 && offset + 4 <= buf.len() {
            // Type A (IPv4)
            let ip = format!("{}.{}.{}.{}", buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3]);
            ips.push(ip);
        }

        offset += rdlength;
    }

    Ok(ips)
}

/// Resolve a hostname against a specific remote upstream DNS server via UDP
pub async fn resolve_via_upstream(hostname: &str, resolver_name: &str, resolver_ip: &str) -> ResolverResult {
    let start = Instant::now();
    let query = build_dns_query(hostname, 0x1234);

    let target_addr: Result<SocketAddr, _> = format!("{}:53", resolver_ip).parse();
    let target_addr = match target_addr {
        Ok(a) => a,
        Err(e) => {
            return ResolverResult {
                resolver_name: resolver_name.to_string(),
                resolver_ip: resolver_ip.to_string(),
                success: false,
                rtt_ms: None,
                resolved_ips: Vec::new(),
                error: Some(format!("Invalid resolver IP: {}", e)),
            };
        }
    };

    let bind_addr = if target_addr.is_ipv6() { "[::]:0" } else { "0.0.0.0:0" };
    let socket = match UdpSocket::bind(bind_addr).await {
        Ok(s) => s,
        Err(e) => {
            return ResolverResult {
                resolver_name: resolver_name.to_string(),
                resolver_ip: resolver_ip.to_string(),
                success: false,
                rtt_ms: None,
                resolved_ips: Vec::new(),
                error: Some(format!("Socket bind error: {}", e)),
            };
        }
    };

    if let Err(e) = socket.send_to(&query, target_addr).await {
        return ResolverResult {
            resolver_name: resolver_name.to_string(),
            resolver_ip: resolver_ip.to_string(),
            success: false,
            rtt_ms: None,
            resolved_ips: Vec::new(),
            error: Some(format!("UDP send error: {}", e)),
        };
    }

    let mut buf = [0u8; 1024];
    let recv_timeout = tokio::time::timeout(Duration::from_millis(2000), socket.recv_from(&mut buf)).await;

    match recv_timeout {
        Ok(Ok((bytes_recvd, _src))) => {
            let rtt = start.elapsed().as_millis() as u64;
            match parse_dns_response(&buf[..bytes_recvd]) {
                Ok(ips) => {
                    let success = !ips.is_empty();
                    ResolverResult {
                        resolver_name: resolver_name.to_string(),
                        resolver_ip: resolver_ip.to_string(),
                        success,
                        rtt_ms: Some(rtt),
                        resolved_ips: ips,
                        error: if success { None } else { Some("No A records in reply".to_string()) },
                    }
                }
                Err(err) => ResolverResult {
                    resolver_name: resolver_name.to_string(),
                    resolver_ip: resolver_ip.to_string(),
                    success: false,
                    rtt_ms: Some(rtt),
                    resolved_ips: Vec::new(),
                    error: Some(err),
                },
            }
        }
        Ok(Err(e)) => ResolverResult {
            resolver_name: resolver_name.to_string(),
            resolver_ip: resolver_ip.to_string(),
            success: false,
            rtt_ms: None,
            resolved_ips: Vec::new(),
            error: Some(format!("UDP recv error: {}", e)),
        },
        Err(_) => ResolverResult {
            resolver_name: resolver_name.to_string(),
            resolver_ip: resolver_ip.to_string(),
            success: false,
            rtt_ms: None,
            resolved_ips: Vec::new(),
            error: Some("DNS query timed out (2000ms)".to_string()),
        },
    }
}

/// Resolve domain via Cloudflare DNS-over-HTTPS (DoH)
pub async fn resolve_via_doh(hostname: &str, proxy_url: Option<&str>) -> ResolverResult {
    let start = Instant::now();
    let url = format!("https://cloudflare-dns.com/dns-query?name={}&type=A", hostname);

    let mut builder = reqwest::Client::builder().timeout(Duration::from_millis(3000));
    if let Some(p) = proxy_url {
        if let Ok(proxy) = reqwest::Proxy::all(p) {
            builder = builder.proxy(proxy);
        }
    }

    let client = match builder.build() {
        Ok(c) => c,
        Err(e) => {
            return ResolverResult {
                resolver_name: "Cloudflare DoH".to_string(),
                resolver_ip: "cloudflare-dns.com".to_string(),
                success: false,
                rtt_ms: None,
                resolved_ips: Vec::new(),
                error: Some(format!("Client build error: {}", e)),
            };
        }
    };

    match client.get(&url).header("accept", "application/dns-json").send().await {
        Ok(resp) => {
            let rtt = start.elapsed().as_millis() as u64;
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                let mut ips = Vec::new();
                if let Some(answers) = json["Answer"].as_array() {
                    for ans in answers {
                        if ans["type"].as_u64() == Some(1) {
                            if let Some(ip_str) = ans["data"].as_str() {
                                ips.push(ip_str.to_string());
                            }
                        }
                    }
                }
                let success = !ips.is_empty();
                ResolverResult {
                    resolver_name: "Cloudflare DoH".to_string(),
                    resolver_ip: "1.1.1.1 (HTTPS)".to_string(),
                    success,
                    rtt_ms: Some(rtt),
                    resolved_ips: ips,
                    error: if success { None } else { Some("No A records in DoH answer".to_string()) },
                }
            } else {
                ResolverResult {
                    resolver_name: "Cloudflare DoH".to_string(),
                    resolver_ip: "1.1.1.1 (HTTPS)".to_string(),
                    success: false,
                    rtt_ms: Some(rtt),
                    resolved_ips: Vec::new(),
                    error: Some("Failed to parse DoH JSON response".to_string()),
                }
            }
        }
        Err(e) => ResolverResult {
            resolver_name: "Cloudflare DoH".to_string(),
            resolver_ip: "1.1.1.1 (HTTPS)".to_string(),
            success: false,
            rtt_ms: None,
            resolved_ips: Vec::new(),
            error: Some(format!("DoH request failed: {}", e)),
        },
    }
}

/// Compare DNS resolutions for target domains
pub async fn diagnose_dns_targets(hostnames: &[&str]) -> DnsDiagnostics {
    let mut items = Vec::new();
    let mut total_latency = 0u64;
    let mut latency_count = 0u64;
    let mut any_tampering = false;

    // Detect if local proxy is active for DoH fallback
    let wininet = crate::diag::proxy::get_wininet_proxy();
    let proxy_ref = if wininet.enabled { wininet.server.as_deref() } else { None };

    for &host in hostnames {
        // Concurrently run system and upstream queries
        let (sys, cf_raw, google, quad9) = tokio::join!(
            resolve_system_dns(host),
            resolve_via_upstream(host, "Cloudflare", "1.1.1.1"),
            resolve_via_upstream(host, "Google", "8.8.8.8"),
            resolve_via_upstream(host, "Quad9", "9.9.9.9")
        );

        // If direct UDP 53 to Cloudflare timed out (common ISP filtering), use DoH
        let cf = if !cf_raw.success {
            resolve_via_doh(host, proxy_ref).await
        } else {
            cf_raw
        };

        if let Some(rtt) = sys.rtt_ms {
            total_latency += rtt;
            latency_count += 1;
        }

        // Check if System DNS resolved to a known ISP blocking stub or redirect
        let is_known_isp_stub = sys.resolved_ips.iter().any(|ip| {
            ip == "185.149.145.6" || ip == "127.0.0.1" || ip == "0.0.0.0"
        });

        // Evaluate tampering/blocking:
        // Case 1: System DNS fails, but Cloudflare or Google succeeds
        let is_blocked = (!sys.success) && (cf.success || google.success);

        // Case 2: Divergence (System returns an IP that none of the public resolvers returned, like a captive portal or ISP landing page)
        let mut divergence = is_known_isp_stub;
        if sys.success && (cf.success || google.success) && !divergence {
            let sys_ip = &sys.resolved_ips;
            let public_ips: Vec<&String> = cf.resolved_ips.iter().chain(google.resolved_ips.iter()).collect();
            let matches = sys_ip.iter().any(|ip| public_ips.contains(&ip));
            if !matches && !public_ips.is_empty() {
                divergence = true;
            }
        }

        let is_tampered_or_blocked = is_blocked || divergence;
        if is_tampered_or_blocked {
            any_tampering = true;
        }

        let diagnosis_note = if is_known_isp_stub {
            format!("Системный DNS подменяет домен {} на IP-заглушку провайдера ({:?}).", host, sys.resolved_ips)
        } else if is_blocked {
            format!("Системный DNS не может разрешить {}, но публичные DNS успешно возвращают IP. Вероятна блокировка провайдером.", host)
        } else if divergence {
            format!("Обнаружено расхождение: Системный DNS возвращает IP {:?}, отличный от публичных серверов. Возможен DNS-спуфинг или редирект провайдера.", sys.resolved_ips)
        } else if sys.success {
            format!("DNS разрешается штатно ({:?}, RTT {}ms).", sys.resolved_ips, sys.rtt_ms.unwrap_or(0))
        } else {
            format!("Не удалось разрешить ни через системный, ни через публичные DNS. Проверьте интернет-соединение.")
        };

        items.push(DnsComparisonItem {
            hostname: host.to_string(),
            system_result: sys,
            cloudflare_result: cf,
            google_result: google,
            quad9_result: quad9,
            is_tampered_or_blocked,
            divergence_detected: divergence,
            diagnosis_note,
        });
    }

    let avg_latency = if latency_count > 0 { total_latency / latency_count } else { 0 };
    let system_dns_working = items.iter().any(|i| i.system_result.success);

    DnsDiagnostics {
        system_dns_working,
        average_dns_latency_ms: avg_latency,
        items,
        any_tampering,
    }
}
