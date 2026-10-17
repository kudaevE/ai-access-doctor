use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualAdapter {
    pub name: String,
    pub description: String,
    pub status: String,
    pub is_active: bool,
    pub vpn_type: String,
    pub ip_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteItem {
    pub destination: String,
    pub netmask: String,
    pub gateway: String,
    pub interface_name: String,
    pub metric: u32,
    pub is_default_gateway: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnDiagnostics {
    pub active_virtual_adapters: Vec<VirtualAdapter>,
    pub default_routes: Vec<RouteItem>,
    pub vpn_tunnel_detected: bool,
    pub has_routing_collision: bool,
    pub analysis_summary: String,
}

/// Classify virtual adapter by description or name
fn identify_vpn_type(desc: &str, name: &str) -> String {
    let combined = format!("{} {}", desc, name).to_lowercase();
    if combined.contains("wireguard") {
        "WireGuard".to_string()
    } else if combined.contains("wintun") {
        "Wintun Userspace Tunnel".to_string()
    } else if combined.contains("tailscale") {
        "Tailscale Mesh VPN".to_string()
    } else if combined.contains("cloudflare") || combined.contains("warp") {
        "Cloudflare WARP".to_string()
    } else if combined.contains("tap-windows") || combined.contains("openvpn") {
        "OpenVPN / TAP Adapter".to_string()
    } else if combined.contains("zerotier") {
        "ZeroTier One".to_string()
    } else if combined.contains("sing-box") || combined.contains("tun") {
        "TUN Virtual Tunnel".to_string()
    } else {
        "Generic Virtual Adapter".to_string()
    }
}

/// Enumerate Windows virtual network adapters and routing entries
pub fn diagnose_vpn() -> VpnDiagnostics {
    let mut active_virtual_adapters = Vec::new();
    let mut default_routes = Vec::new();
    let mut vpn_tunnel_detected = false;

    #[cfg(windows)]
    {
        // 1. Query network adapters (silently without console window)
        let adapter_output = super::silent_command("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-NetAdapter | Select-Object Name, InterfaceDescription, Status, Virtual | ConvertTo-Json",
            ])
            .output();

        if let Ok(out) = adapter_output {
            let str_val = String::from_utf8_lossy(&out.stdout);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&str_val) {
                let items = if val.is_array() {
                    val.as_array().cloned().unwrap_or_default()
                } else if val.is_object() {
                    vec![val]
                } else {
                    vec![]
                };

                for item in items {
                    let name = item["Name"].as_str().unwrap_or("").to_string();
                    let desc = item["InterfaceDescription"].as_str().unwrap_or("").to_string();
                    let status = item["Status"].as_str().unwrap_or("").to_string();
                    let is_virtual = item["Virtual"].as_bool().unwrap_or(false)
                        || desc.to_lowercase().contains("virtual")
                        || desc.to_lowercase().contains("tap")
                        || desc.to_lowercase().contains("tun")
                        || desc.to_lowercase().contains("wintun")
                        || desc.to_lowercase().contains("wireguard")
                        || desc.to_lowercase().contains("tailscale")
                        || desc.to_lowercase().contains("warp");

                    if is_virtual {
                        let is_active = status.eq_ignore_ascii_case("Up");
                        if is_active {
                            vpn_tunnel_detected = true;
                        }

                        let vpn_type = identify_vpn_type(&desc, &name);
                        active_virtual_adapters.push(VirtualAdapter {
                            name,
                            description: desc,
                            status,
                            is_active,
                            vpn_type,
                            ip_addresses: Vec::new(),
                        });
                    }
                }
            }
        }

        // 2. Query default IPv4 routing table via Get-NetRoute (silently without console window)
        let route_output = super::silent_command("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue | Select-Object DestinationPrefix, NextHop, InterfaceAlias, RouteMetric | ConvertTo-Json",
            ])
            .output();

        if let Ok(out) = route_output {
            let str_val = String::from_utf8_lossy(&out.stdout);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&str_val) {
                let items = if val.is_array() {
                    val.as_array().cloned().unwrap_or_default()
                } else if val.is_object() {
                    vec![val]
                } else {
                    vec![]
                };

                for item in items {
                    let dest = item["DestinationPrefix"].as_str().unwrap_or("0.0.0.0/0").to_string();
                    let gateway = item["NextHop"].as_str().unwrap_or("").to_string();
                    let iface = item["InterfaceAlias"].as_str().unwrap_or("").to_string();
                    let metric = item["RouteMetric"].as_u64().unwrap_or(0) as u32;

                    default_routes.push(RouteItem {
                        destination: dest,
                        netmask: "0.0.0.0".to_string(),
                        gateway,
                        interface_name: iface,
                        metric,
                        is_default_gateway: true,
                    });
                }
            }
        }
    }

    // Routing collision detection: Multiple default routes with identical or very close metrics
    let has_routing_collision = default_routes.len() > 1 && {
        let metrics: Vec<u32> = default_routes.iter().map(|r| r.metric).collect();
        metrics[0] == metrics[1]
    };

    let analysis_summary = if has_routing_collision {
        "Обнаружен конфликт маршрутов: несколько адаптеров имеют одинаковый метрик для маршрута по умолчанию 0.0.0.0/0. Трафик может сбрасываться.".to_string()
    } else if vpn_tunnel_detected {
        format!("Активен VPN-туннель (найдено активных виртуальных адаптеров: {}). Весь трафик направляется через защищённый туннель.", active_virtual_adapters.iter().filter(|a| a.is_active).count())
    } else if !active_virtual_adapters.is_empty() {
        "Обнаружены установленные VPN-адаптеры, но в данный момент они отключены. Трафик идёт через физический сетевой интерфейс.".to_string()
    } else {
        "Виртуальные VPN-адаптеры не обнаружены. Сетевое подключение осуществляется напрямую через физический адаптер.".to_string()
    };

    VpnDiagnostics {
        active_virtual_adapters,
        default_routes,
        vpn_tunnel_detected,
        has_routing_collision,
        analysis_summary,
    }
}
