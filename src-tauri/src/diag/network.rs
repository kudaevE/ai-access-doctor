use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceInfo {
    pub name: String,
    pub description: String,
    pub status: String,
    pub is_up: bool,
    pub mac: Option<String>,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    pub gateways: Vec<String>,
    pub dns_servers: Vec<String>,
    pub is_virtual: bool,
    pub adapter_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualStackStatus {
    pub ipv4_working: bool,
    pub ipv4_rtt_ms: Option<u64>,
    pub ipv6_working: bool,
    pub ipv6_rtt_ms: Option<u64>,
    pub preferred_family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpProbeResult {
    pub target: String,
    pub port: u16,
    pub success: bool,
    pub rtt_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiagnostics {
    pub interfaces: Vec<InterfaceInfo>,
    pub dual_stack: DualStackStatus,
    pub primary_gateway: Option<String>,
    pub primary_interface: Option<String>,
    pub default_dns: Vec<String>,
}

/// Perform TCP connection probe to measure reachability and round-trip latency
pub async fn probe_tcp(host: &str, port: u16, timeout_duration: Duration) -> TcpProbeResult {
    let start = Instant::now();
    let addr_str = format!("{}:{}", host, port);

    // Resolve address
    let socket_addrs = match tokio::net::lookup_host(&addr_str).await {
        Ok(addrs) => addrs.collect::<Vec<_>>(),
        Err(e) => {
            return TcpProbeResult {
                target: host.to_string(),
                port,
                success: false,
                rtt_ms: None,
                error: Some(format!("DNS lookup failed: {}", e)),
            };
        }
    };

    if socket_addrs.is_empty() {
        return TcpProbeResult {
            target: host.to_string(),
            port,
            success: false,
            rtt_ms: None,
            error: Some("No IP addresses found for target".to_string()),
        };
    }

    // Try connecting to first available socket address with timeout
    let connect_res = tokio::time::timeout(timeout_duration, TcpStream::connect(&socket_addrs[0])).await;

    match connect_res {
        Ok(Ok(_stream)) => {
            let rtt = start.elapsed().as_millis() as u64;
            TcpProbeResult {
                target: host.to_string(),
                port,
                success: true,
                rtt_ms: Some(rtt),
                error: None,
            }
        }
        Ok(Err(e)) => {
            TcpProbeResult {
                target: host.to_string(),
                port,
                success: false,
                rtt_ms: None,
                error: Some(format!("Connection refused or reset: {}", e)),
            }
        }
        Err(_) => {
            TcpProbeResult {
                target: host.to_string(),
                port,
                success: false,
                rtt_ms: None,
                error: Some(format!("Connection timed out after {}ms", timeout_duration.as_millis())),
            }
        }
    }
}

/// Check dual-stack IPv4 and IPv6 public connectivity
pub async fn check_dual_stack() -> DualStackStatus {
    // IPv4 probe: Cloudflare DNS (1.1.1.1:53) or Google (8.8.8.8:53)
    let ipv4_start = Instant::now();
    let ipv4_target: SocketAddr = "1.1.1.1:53".parse().unwrap();
    let ipv4_res = tokio::time::timeout(Duration::from_millis(1500), TcpStream::connect(ipv4_target)).await;
    let (ipv4_working, ipv4_rtt_ms) = match ipv4_res {
        Ok(Ok(_)) => (true, Some(ipv4_start.elapsed().as_millis() as u64)),
        _ => (false, None),
    };

    // IPv6 probe: Cloudflare IPv6 DNS ([2606:4700:4700::1111]:53)
    let ipv6_start = Instant::now();
    let ipv6_target: Result<SocketAddr, _> = "[2606:4700:4700::1111]:53".parse();
    let (ipv6_working, ipv6_rtt_ms) = match ipv6_target {
        Ok(addr) => {
            let res = tokio::time::timeout(Duration::from_millis(1500), TcpStream::connect(addr)).await;
            match res {
                Ok(Ok(_)) => (true, Some(ipv6_start.elapsed().as_millis() as u64)),
                _ => (false, None),
            }
        }
        Err(_) => (false, None),
    };

    let preferred_family = if ipv6_working && ipv4_working {
        if ipv6_rtt_ms.unwrap_or(u64::MAX) < ipv4_rtt_ms.unwrap_or(u64::MAX) {
            "IPv6 (Dual-Stack, IPv6 Faster)".to_string()
        } else {
            "IPv4 (Dual-Stack, IPv4 Faster)".to_string()
        }
    } else if ipv4_working {
        "IPv4-Only".to_string()
    } else if ipv6_working {
        "IPv6-Only".to_string()
    } else {
        "No Direct Internet Connectivity".to_string()
    };

    DualStackStatus {
        ipv4_working,
        ipv4_rtt_ms,
        ipv6_working,
        ipv6_rtt_ms,
        preferred_family,
    }
}

/// Enumerate Windows network adapters, IP addresses, Gateways and DNS servers
pub fn get_network_interfaces() -> Vec<InterfaceInfo> {
    let mut interfaces = Vec::new();

    #[cfg(windows)]
    {
        // Use PowerShell Get-NetAdapter and Get-NetIPConfiguration for comprehensive Windows detail
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "$adapters = Get-NetAdapter | Select-Object Name, InterfaceDescription, Status, MacAddress, Virtual; $configs = Get-NetIPConfiguration; @{ Adapters = $adapters; Configs = ($configs | Select-Object InterfaceAlias, IPv4Address, IPv6Address, IPv4DefaultGateway, DNSServer) } | ConvertTo-Json -Depth 4",
            ])
            .output();

        if let Ok(out) = output {
            let stdout_str = String::from_utf8_lossy(&out.stdout);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&stdout_str) {
                if let Some(adapters) = val.get("Adapters").and_then(|a| a.as_array()) {
                    let configs = val.get("Configs").and_then(|c| c.as_array()).cloned().unwrap_or_default();

                    for adapter in adapters {
                        let name = adapter["Name"].as_str().unwrap_or("Unknown").to_string();
                        let desc = adapter["InterfaceDescription"].as_str().unwrap_or("").to_string();
                        let status = adapter["Status"].as_str().unwrap_or("Unknown").to_string();
                        let is_up = status.eq_ignore_ascii_case("Up");
                        let mac = adapter["MacAddress"].as_str().map(|s| s.to_string());
                        let is_virtual = adapter["Virtual"].as_bool().unwrap_or(false)
                            || desc.to_lowercase().contains("virtual")
                            || desc.to_lowercase().contains("tap")
                            || desc.to_lowercase().contains("tun")
                            || desc.to_lowercase().contains("wintun")
                            || desc.to_lowercase().contains("wireguard")
                            || desc.to_lowercase().contains("tailscale")
                            || desc.to_lowercase().contains("warp")
                            || desc.to_lowercase().contains("vpn");

                        let mut ipv4_list = Vec::new();
                        let mut ipv6_list = Vec::new();
                        let mut gateways = Vec::new();
                        let mut dns_servers = Vec::new();

                        // Match config by InterfaceAlias
                        for cfg in &configs {
                            if cfg["InterfaceAlias"].as_str() == Some(&name) {
                                // IPv4
                                if let Some(ips) = cfg["IPv4Address"].as_array() {
                                    for ip in ips {
                                        if let Some(s) = ip.as_str() {
                                            ipv4_list.push(s.to_string());
                                        } else if let Some(s) = ip["IPAddress"].as_str() {
                                            ipv4_list.push(s.to_string());
                                        }
                                    }
                                } else if let Some(ip) = cfg["IPv4Address"].as_str() {
                                    ipv4_list.push(ip.to_string());
                                }

                                // IPv6
                                if let Some(ips) = cfg["IPv6Address"].as_array() {
                                    for ip in ips {
                                        if let Some(s) = ip.as_str() {
                                            ipv6_list.push(s.to_string());
                                        } else if let Some(s) = ip["IPAddress"].as_str() {
                                            ipv6_list.push(s.to_string());
                                        }
                                    }
                                } else if let Some(ip) = cfg["IPv6Address"].as_str() {
                                    ipv6_list.push(ip.to_string());
                                }

                                // Gateways
                                if let Some(gws) = cfg["IPv4DefaultGateway"].as_array() {
                                    for gw in gws {
                                        if let Some(s) = gw["NextHop"].as_str() {
                                            gateways.push(s.to_string());
                                        }
                                    }
                                } else if let Some(gw) = cfg["IPv4DefaultGateway"]["NextHop"].as_str() {
                                    gateways.push(gw.to_string());
                                }

                                // DNS Servers
                                if let Some(servers) = cfg["DNSServer"].as_array() {
                                    for srv in servers {
                                        if let Some(s) = srv["ServerAddresses"].as_array() {
                                            for addr in s {
                                                if let Some(ip_str) = addr.as_str() {
                                                    dns_servers.push(ip_str.to_string());
                                                }
                                            }
                                        }
                                    }
                                } else if let Some(srv) = cfg["DNSServer"]["ServerAddresses"].as_array() {
                                    for addr in srv {
                                        if let Some(ip_str) = addr.as_str() {
                                            dns_servers.push(ip_str.to_string());
                                        }
                                    }
                                }
                            }
                        }

                        let adapter_type = if is_virtual {
                            "Virtual / VPN Adapter".to_string()
                        } else if desc.to_lowercase().contains("wi-fi") || desc.to_lowercase().contains("wireless") {
                            "Wi-Fi".to_string()
                        } else {
                            "Ethernet".to_string()
                        };

                        interfaces.push(InterfaceInfo {
                            name,
                            description: desc,
                            status,
                            is_up,
                            mac,
                            ipv4: ipv4_list,
                            ipv6: ipv6_list,
                            gateways,
                            dns_servers,
                            is_virtual,
                            adapter_type,
                        });
                    }
                }
            }
        }
    }

    if interfaces.is_empty() {
        // Fallback default interface
        interfaces.push(InterfaceInfo {
            name: "Default Adapter".to_string(),
            description: "System Primary Network Adapter".to_string(),
            status: "Up".to_string(),
            is_up: true,
            mac: None,
            ipv4: vec!["127.0.0.1".to_string()],
            ipv6: vec!["::1".to_string()],
            gateways: vec![],
            dns_servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
            is_virtual: false,
            adapter_type: "Ethernet".to_string(),
        });
    }

    interfaces
}

/// Run full network diagnostic sweep
pub async fn diagnose_network() -> NetworkDiagnostics {
    let interfaces = get_network_interfaces();
    let dual_stack = check_dual_stack().await;

    let mut primary_gateway = None;
    let mut primary_interface = None;
    let mut default_dns = Vec::new();

    for iface in &interfaces {
        if iface.is_up && !iface.gateways.is_empty() {
            primary_gateway = Some(iface.gateways[0].clone());
            primary_interface = Some(iface.name.clone());
            default_dns.extend(iface.dns_servers.clone());
            break;
        }
    }

    default_dns.dedup();

    NetworkDiagnostics {
        interfaces,
        dual_stack,
        primary_gateway,
        primary_interface,
        default_dns,
    }
}
