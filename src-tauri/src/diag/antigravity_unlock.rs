use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NrptRuleSummary {
    pub name: String,
    pub namespace: Vec<String>,
    pub name_servers: Vec<String>,
    pub comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigravityDoctorReport {
    pub ide_installed: bool,
    pub ide_path: Option<String>,
    pub ide_version: Option<String>,
    pub language_server_installed: bool,
    pub language_server_path: Option<String>,
    pub language_server_size_mb: Option<f64>,
    pub ag_dns_running: bool,
    pub ag_dns_pid: Option<u32>,
    pub loopback_relay_ready: bool, // 127.65.71.1:443 listening
    pub nrpt_active: bool,
    pub nrpt_rules: Vec<NrptRuleSummary>,
    pub ipv4_preferred: bool,
    pub cloudcode_direct_status: String,
    pub cloudcode_rtt_ms: Option<u64>,
    pub accounts_status: String,
    pub accounts_ip: Option<String>,
    pub ag_ls_proxy_env: Option<String>,
    pub overall_status_ru: String,
    pub status_badge: String, // "OPTIMAL", "WARNING", "DISABLED"
}

/// Helper to get local app data path
fn get_local_appdata() -> Option<PathBuf> {
    std::env::var("LOCALAPPDATA").ok().map(PathBuf::from)
}

/// Helper to get program data path
fn get_program_data() -> PathBuf {
    std::env::var("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData"))
}

/// Check if Antigravity is installed and locate main executables
pub fn inspect_antigravity_installation() -> (bool, Option<String>, Option<String>, bool, Option<String>, Option<f64>) {
    let mut ide_installed = false;
    let mut ide_path = None;
    let mut ide_version = None;
    let mut ls_installed = false;
    let mut ls_path = None;
    let mut ls_size_mb = None;

    if let Some(local_app) = get_local_appdata() {
        // Check standard directory: %LOCALAPPDATA%\Programs\antigravity
        let base_dir = local_app.join("Programs").join("antigravity");
        let ide_exe = base_dir.join("Antigravity.exe");
        if ide_exe.exists() {
            ide_installed = true;
            ide_path = Some(ide_exe.to_string_lossy().to_string());
            ide_version = Some("2.23+ Native".to_string());
        }

        // Check language_server.exe in resources\bin\
        let ls_exe = base_dir.join("resources").join("bin").join("language_server.exe");
        if ls_exe.exists() {
            ls_installed = true;
            ls_path = Some(ls_exe.to_string_lossy().to_string());
            if let Ok(meta) = std::fs::metadata(&ls_exe) {
                ls_size_mb = Some((meta.len() as f64 / (1024.0 * 1024.0) * 10.0).round() / 10.0);
            }
        }
    }

    (ide_installed, ide_path, ide_version, ls_installed, ls_path, ls_size_mb)
}

/// Query active Windows NRPT rules via PowerShell
pub fn get_active_nrpt_rules() -> Vec<NrptRuleSummary> {
    let script = r#"Get-DnsClientNrptRule -ErrorAction SilentlyContinue | Select-Object Name, Namespace, NameServers, Comment | ConvertTo-Json -Compress"#;
    let output = crate::diag::silent_command("powershell")
        .args(["-NoProfile", "-Command", script])
        .output();

    let mut rules = Vec::new();

    if let Ok(out) = output {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !text.is_empty() && text != "null" {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                    let items = if let Some(arr) = val.as_array() {
                        arr.clone()
                    } else if val.is_object() {
                        vec![val]
                    } else {
                        Vec::new()
                    };

                    for item in items {
                        let name = item["Name"].as_str().unwrap_or("").to_string();
                        let comment = item["Comment"].as_str().unwrap_or("").to_string();

                        let mut namespace = Vec::new();
                        if let Some(ns_arr) = item["Namespace"].as_array() {
                            for ns in ns_arr {
                                if let Some(s) = ns.as_str() {
                                    namespace.push(s.to_string());
                                }
                            }
                        } else if let Some(ns_str) = item["Namespace"].as_str() {
                            namespace.push(ns_str.to_string());
                        }

                        let mut name_servers = Vec::new();
                        if let Some(servers) = item["NameServers"].as_array() {
                            for s in servers {
                                if let Some(ip) = s.as_str() {
                                    name_servers.push(ip.to_string());
                                }
                            }
                        } else if let Some(val_obj) = item["NameServers"].get("value") {
                            if let Some(arr) = val_obj.as_array() {
                                for s in arr {
                                    if let Some(ip) = s.as_str() {
                                        name_servers.push(ip.to_string());
                                    }
                                }
                            }
                        } else if let Some(s) = item["NameServers"].as_str() {
                            name_servers.push(s.to_string());
                        }

                        rules.push(NrptRuleSummary {
                            name,
                            namespace,
                            name_servers,
                            comment,
                        });
                    }
                }
            }
        }
    }

    rules
}

/// Check if ag_dns.exe process is running
pub fn check_ag_dns_running() -> (bool, Option<u32>) {
    let output = crate::diag::silent_command("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Process -Name ag_dns -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id -First 1",
        ])
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if let Ok(pid) = text.parse::<u32>() {
            return (true, Some(pid));
        }
    }

    (false, None)
}

/// Check if loopback relay on 127.65.71.1:443 is accepting connections
pub fn check_loopback_relay_ready() -> bool {
    let script = r#"try { $t = New-Object System.Net.Sockets.TcpClient; $t.Connect('127.65.71.1', 443); $t.Connected; $t.Close() } catch { $false }"#;
    let output = crate::diag::silent_command("powershell")
        .args(["-NoProfile", "-Command", script])
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
        return text == "true";
    }

    false
}

/// Check if IPv4 prefix policy has higher priority than IPv6
pub fn check_ipv4_preferred() -> bool {
    let output = crate::diag::silent_command("netsh")
        .args(["interface", "ipv6", "show", "prefixpolicies"])
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        // Look for line containing ::ffff:0:0/96 and check if its precedence > 40
        for line in text.lines() {
            if line.contains("::ffff:0:0/96") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(prec_str) = parts.first() {
                    if let Ok(prec) = prec_str.parse::<u32>() {
                        return prec >= 40;
                    }
                }
            }
        }
    }

    false
}

/// Execute elevated PowerShell command safely without flashing console windows
fn execute_elevated_powershell(command: &str) -> Result<(), String> {
    let script = format!(
        "Start-Process powershell -Verb RunAs -Wait -WindowStyle Hidden -ArgumentList \"-NoProfile -Command {}\"",
        command
    );
    let output = crate::diag::silent_command("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map_err(|e| format!("Не удалось вызвать повышение привилегий: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Операция отменена пользователем или завершилась ошибкой: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// Comprehensive audit of Google Antigravity connectivity in Russia / CIS
pub async fn audit_antigravity_doctor() -> AntigravityDoctorReport {
    let (ide_installed, ide_path, ide_version, ls_installed, ls_path, ls_size_mb) =
        inspect_antigravity_installation();

    let nrpt_rules = get_active_nrpt_rules();
    let has_google_nrpt = nrpt_rules.iter().any(|r| {
        r.namespace
            .iter()
            .any(|ns| ns.contains("cloudcode-pa.googleapis.com") || ns.contains("googleapis.com"))
    });

    let (ag_dns_running, ag_dns_pid) = check_ag_dns_running();
    let loopback_relay_ready = check_loopback_relay_ready();
    let ipv4_preferred = check_ipv4_preferred();

    let ag_ls_proxy_env = std::env::var("AG_LS_PROXY").ok();

    // Probe accounts.google.com DNS
    let mut accounts_status = "Проверка...".to_string();
    let mut accounts_ip = None;

    if let Ok(addrs) = tokio::net::lookup_host(("accounts.google.com", 443u16)).await {
        let ips: Vec<String> = addrs.map(|a| a.ip().to_string()).collect();
        if let Some(first_ip) = ips.first() {
            accounts_ip = Some(first_ip.clone());
            if first_ip == "185.149.145.6" {
                accounts_status = "БЛОКИРОВКА: Провайдер подменяет адрес на заглушку (185.149.145.6)".to_string();
            } else {
                accounts_status = format!("Разрешается штатно ({})", ips.join(", "));
            }
        }
    } else {
        accounts_status = "Ошибка DNS-разрешения".to_string();
    }

    // Direct probe of cloudcode-pa.googleapis.com
    let start = Instant::now();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(3500))
        .build();

    let mut cloudcode_direct_status = "Недоступен".to_string();
    let mut cloudcode_rtt_ms = None;

    if let Ok(c) = client {
        match c.get("https://cloudcode-pa.googleapis.com").send().await {
            Ok(resp) => {
                cloudcode_rtt_ms = Some(start.elapsed().as_millis() as u64);
                let status = resp.status().as_u16();
                if status == 200 || status == 404 {
                    cloudcode_direct_status = format!("Доступен (HTTP {}, Google PA отвечает)", status);
                } else if status == 400 {
                    cloudcode_direct_status = "REGION-400 (Google блокирует запросы из региона РФ/СНГ)".to_string();
                } else {
                    cloudcode_direct_status = format!("Ответ HTTP {}", status);
                }
            }
            Err(e) => {
                if loopback_relay_ready {
                    cloudcode_direct_status = "Туннель активен на 127.65.71.1:443 (локальный шлюз готов)".to_string();
                } else {
                    cloudcode_direct_status = format!("Ошибка соединения: {}", e);
                }
            }
        }
    }

    // Overall verdict
    let (overall_status_ru, status_badge) = if has_google_nrpt && (ag_dns_running || loopback_relay_ready) {
        (
            "Обход для Antigravity АКТИВЕН: NRPT перенаправляет запросы к Google Cloud Code, туннель запущен."
                .to_string(),
            "OPTIMAL".to_string(),
        )
    } else if has_google_nrpt {
        (
            "Правила NRPT настроены, но локальный процесс туннеля не запущен. Рекомендуется нажать «Включить обход»."
                .to_string(),
            "WARNING".to_string(),
        )
    } else {
        (
            "Обход Antigravity отключен. Запросы к Google CloudCode идут напрямую и могут блокироваться провайдером или вызывать region-400."
                .to_string(),
            "DISABLED".to_string(),
        )
    };

    AntigravityDoctorReport {
        ide_installed,
        ide_path,
        ide_version,
        language_server_installed: ls_installed,
        language_server_path: ls_path,
        language_server_size_mb: ls_size_mb,
        ag_dns_running,
        ag_dns_pid,
        loopback_relay_ready,
        nrpt_active: has_google_nrpt,
        nrpt_rules,
        ipv4_preferred,
        cloudcode_direct_status,
        cloudcode_rtt_ms,
        accounts_status,
        accounts_ip,
        ag_ls_proxy_env,
        overall_status_ru,
        status_badge,
    }
}

/// Enable 1-Click Antigravity Unlock for Windows
pub fn apply_antigravity_unlock_profile(profile_type: &str) -> Result<String, String> {
    // 1. Snapshot baseline if not already taken
    let _ = crate::diag::free_hub::ensure_snapshot();

    // 2. Configure NRPT rules according to selected profile
    match profile_type {
        "ag_relay" => {
            // Proven AG Unlocker rules (matching AG_2.23.1.exe)
            let cmd = r#"
                Get-DnsClientNrptRule -ErrorAction SilentlyContinue | Where-Object { $_.Comment -like '*AG_*' -or $_.Comment -like '*AI_ACCESS_*' } | Remove-DnsClientNrptRule -Force -ErrorAction SilentlyContinue;
                Add-DnsClientNrptRule -Namespace 'cloudcode-pa.googleapis.com' -NameServers '127.0.0.53','83.220.169.155' -Comment 'AI_ACCESS_DOCTOR_NRPT';
                Add-DnsClientNrptRule -Namespace 'daily-cloudcode-pa.googleapis.com' -NameServers '127.0.0.53','83.220.169.155','45.155.204.190' -Comment 'AI_ACCESS_DOCTOR_NRPT';
                netsh interface ipv6 set prefixpolicy ::ffff:0:0/96 46 4;
                ipconfig /flushdns
            "#;
            execute_elevated_powershell(cmd)?;

            // If AG Unlocker ag_dns.exe exists in ProgramData, ensure it is running
            let ag_dns_path = get_program_data().join("AGUnlocker").join("ag_dns.exe");
            if ag_dns_path.exists() {
                let (running, _) = check_ag_dns_running();
                if !running {
                    let _ = crate::diag::silent_command(ag_dns_path.to_str().unwrap_or(""))
                        .args(["--watchdog"])
                        .spawn();
                }
            }

            Ok("Режим «Antigravity Relay» успешно активирован. Правила NRPT и приоритет IPv4 применены.".to_string())
        }
        "clean_doh" => {
            // Clean direct Cloudflare & Google resolvers for Google AI
            let cmd = r#"
                Get-DnsClientNrptRule -ErrorAction SilentlyContinue | Where-Object { $_.Comment -like '*AG_*' -or $_.Comment -like '*AI_ACCESS_*' } | Remove-DnsClientNrptRule -Force -ErrorAction SilentlyContinue;
                Add-DnsClientNrptRule -Namespace 'cloudcode-pa.googleapis.com' -NameServers '1.1.1.1','8.8.8.8' -Comment 'AI_ACCESS_DOCTOR_NRPT';
                Add-DnsClientNrptRule -Namespace 'daily-cloudcode-pa.googleapis.com' -NameServers '1.1.1.1','8.8.8.8' -Comment 'AI_ACCESS_DOCTOR_NRPT';
                Add-DnsClientNrptRule -Namespace 'generativelanguage.googleapis.com' -NameServers '1.1.1.1','8.8.8.8' -Comment 'AI_ACCESS_DOCTOR_NRPT';
                netsh interface ipv6 set prefixpolicy ::ffff:0:0/96 46 4;
                ipconfig /flushdns
            "#;
            execute_elevated_powershell(cmd)?;
            Ok("Режим «Чистый DoH / Anycast» активирован. Домены Google PA направлены на 1.1.1.1 и 8.8.8.8.".to_string())
        }
        "user_proxy" => {
            // Point Language Server and Windows tools to active local proxy
            let cmd = r#"
                [System.Environment]::SetEnvironmentVariable('AG_LS_PROXY', 'http://127.0.0.1:10809', 'User');
                [System.Environment]::SetEnvironmentVariable('NO_PROXY', 'localhost,127.0.0.1', 'User');
                netsh interface ipv6 set prefixpolicy ::ffff:0:0/96 46 4;
                ipconfig /flushdns
            "#;
            let out = crate::diag::silent_command("powershell")
                .args(["-NoProfile", "-Command", cmd])
                .output()
                .map_err(|e| e.to_string())?;

            if out.status.success() {
                Ok("Переменная AG_LS_PROXY привязана к прокси 127.0.0.1:10809. Language Server будет использовать этот маршрут.".to_string())
            } else {
                Err("Ошибка при установке переменной окружения".to_string())
            }
        }
        _ => Err(format!("Неизвестный профиль: {}", profile_type)),
    }
}

/// Rollback all Antigravity modifications cleanly to Windows defaults
pub fn rollback_antigravity_unlock() -> Result<String, String> {
    let cmd = r#"
        Get-DnsClientNrptRule -ErrorAction SilentlyContinue | Where-Object { $_.Comment -like '*AG_*' -or $_.Comment -like '*AI_ACCESS_*' } | Remove-DnsClientNrptRule -Force -ErrorAction SilentlyContinue;
        netsh interface ipv6 reset;
        ipconfig /flushdns
    "#;
    execute_elevated_powershell(cmd)?;

    // Clear environment variable
    let clear_env = r#"[System.Environment]::SetEnvironmentVariable('AG_LS_PROXY', $null, 'User')"#;
    let _ = crate::diag::silent_command("powershell")
        .args(["-NoProfile", "-Command", clear_env])
        .output();

    // Terminate ag_dns if running
    let _ = crate::diag::silent_command("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Stop-Process -Name ag_dns -Force -ErrorAction SilentlyContinue",
        ])
        .output();

    Ok("Все правила NRPT для Antigravity удалены, стек IPv6 сброшен к стандартам Windows, переменные окружения очищены.".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedProxyEndpoint {
    pub name: String,
    pub url: String,
    pub is_online: bool,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigravityClientSuiteInfo {
    pub is_installed: bool,
    pub exe_path: Option<String>,
    pub is_running: bool,
    pub running_pids: Vec<u32>,
    pub settings_path: String,
    pub settings_configured: bool,
    pub current_proxy_in_settings: Option<String>,
    pub detected_proxies: Vec<DetectedProxyEndpoint>,
    pub latest_oauth_url: Option<String>,
}

pub fn is_port_online(port: u16) -> bool {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok()
}

pub fn get_detected_proxies() -> Vec<DetectedProxyEndpoint> {
    let candidates = [
        ("Dagami Agent Proxy Bridge", 53200),
        ("v2rayN / Xray (HTTP)", 10809),
        ("Clash / Sing-box (Mixed)", 7890),
        ("v2rayN / Xray (SOCKS5)", 10808),
        ("Nekoray (HTTP)", 2081),
        ("Nekoray (SOCKS5)", 2080),
    ];

    let mut result = Vec::new();
    for (name, port) in candidates {
        let online = is_port_online(port);
        let prefix = if name.contains("SOCKS") { "socks5" } else { "http" };
        result.push(DetectedProxyEndpoint {
            name: name.to_string(),
            url: format!("{}://127.0.0.1:{}", prefix, port),
            is_online: online,
            port,
        });
    }

    result
}

pub fn get_antigravity_settings_path() -> PathBuf {
    let appdata = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"));
    appdata.join("Antigravity").join("User").join("settings.json")
}

pub fn extract_latest_oauth_url(content: &str) -> Option<String> {
    let prefix = "https://accounts.google.com/";
    let mut last_found = None;
    for (idx, _) in content.match_indices(prefix) {
        let slice = &content[idx..];
        let end_idx = slice
            .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '<' || c == '>' || c == '\x1b')
            .unwrap_or(slice.len());
        let url = &slice[..end_idx];
        if url.contains("/o/oauth2/") || url.contains("/v3/signin/") || url.contains("/signin/") {
            last_found = Some(url.to_string());
        }
    }
    last_found
}

pub fn get_antigravity_log_oauth_url() -> Option<String> {
    let mut candidate_paths = Vec::new();
    if let Ok(appdata) = std::env::var("APPDATA") {
        candidate_paths.push(PathBuf::from(&appdata).join("Antigravity").join("logs").join("language_server.log"));
    }
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        candidate_paths.push(PathBuf::from(&local_appdata).join("antigravity").join("logs").join("language_server.log"));
    }

    for path in candidate_paths {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Some(url) = extract_latest_oauth_url(&content) {
                    return Some(url);
                }
            }
        }
    }
    None
}

pub fn get_antigravity_client_suite_info() -> AntigravityClientSuiteInfo {
    let (is_installed, ide_path, _, _, _, _) = inspect_antigravity_installation();

    let settings_file = get_antigravity_settings_path();
    let mut settings_configured = false;
    let mut current_proxy = None;

    if settings_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&settings_file) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(p) = val.get("http.proxy").and_then(|v| v.as_str()) {
                    current_proxy = Some(p.to_string());
                }
                if let Some(support) = val.get("http.proxySupport").and_then(|v| v.as_str()) {
                    if support.eq_ignore_ascii_case("override") {
                        settings_configured = true;
                    }
                }
            }
        }
    }

    // Check running processes
    let mut running_pids = Vec::new();
    let script = "Get-Process -Name Antigravity -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id";
    if let Ok(out) = crate::diag::silent_command("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if let Ok(pid) = line.trim().parse::<u32>() {
                running_pids.push(pid);
            }
        }
    }
    let is_running = !running_pids.is_empty();

    let detected_proxies = get_detected_proxies();
    let latest_oauth_url = get_antigravity_log_oauth_url();

    AntigravityClientSuiteInfo {
        is_installed,
        exe_path: ide_path,
        is_running,
        running_pids,
        settings_path: settings_file.to_string_lossy().to_string(),
        settings_configured,
        current_proxy_in_settings: current_proxy,
        detected_proxies,
        latest_oauth_url,
    }
}

pub fn patch_antigravity_settings(proxy_url: &str) -> Result<String, String> {
    let settings_file = get_antigravity_settings_path();
    if let Some(parent) = settings_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut map: serde_json::Map<String, serde_json::Value> = if settings_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&settings_file) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            serde_json::Map::new()
        }
    } else {
        serde_json::Map::new()
    };

    map.insert("http.proxy".to_string(), serde_json::Value::String(proxy_url.to_string()));
    map.insert("http.proxySupport".to_string(), serde_json::Value::String("override".to_string()));
    map.insert("http.proxyStrictSSL".to_string(), serde_json::Value::Bool(false));
    map.insert("http.noProxy".to_string(), serde_json::Value::String("localhost,127.0.0.1,::1".to_string()));

    let formatted = serde_json::to_string_pretty(&map).map_err(|e| e.to_string())?;
    std::fs::write(&settings_file, formatted).map_err(|e| format!("Не удалось сохранить settings.json: {}", e))?;

    Ok(format!("Настройки успешно обновлены: прокси {} прописан с флагом http.proxySupport: override.", proxy_url))
}

pub fn launch_antigravity_with_proxy(proxy_url: &str) -> Result<String, String> {
    let (is_installed, ide_path_opt, _, _, _, _) = inspect_antigravity_installation();
    if !is_installed || ide_path_opt.is_none() {
        return Err("Исполняемый файл Antigravity.exe не найден в системе. Убедитесь, что Antigravity установлена.".to_string());
    }

    let exe_path = ide_path_opt.unwrap();

    // First, ensure settings.json is patched
    let _ = patch_antigravity_settings(proxy_url);

    // Also set user environment variables AGY_PROXY_URL and HTTP_PROXY
    let set_env_script = format!(
        "[System.Environment]::SetEnvironmentVariable('AGY_PROXY_URL', '{}', 'User'); [System.Environment]::SetEnvironmentVariable('HTTP_PROXY', '{}', 'User'); [System.Environment]::SetEnvironmentVariable('HTTPS_PROXY', '{}', 'User')",
        proxy_url, proxy_url, proxy_url
    );
    let _ = crate::diag::silent_command("powershell")
        .args(["-NoProfile", "-Command", &set_env_script])
        .output();

    let proxy_arg = format!("--proxy-server={}", proxy_url);
    let bypass_arg = "--proxy-bypass-list=localhost;127.0.0.1;[::1]";
    let quic_arg = "--disable-quic";

    let mut cmd = std::process::Command::new(&exe_path);
    cmd.args([&proxy_arg, bypass_arg, quic_arg]);
    cmd.env("HTTP_PROXY", proxy_url);
    cmd.env("HTTPS_PROXY", proxy_url);
    cmd.env("ALL_PROXY", proxy_url);
    cmd.env("AGY_PROXY_URL", proxy_url);
    cmd.env("NO_PROXY", "localhost,127.0.0.1,::1");

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }

    match cmd.spawn() {
        Ok(child) => Ok(format!(
            "Antigravity успешно запущена (PID {}) с аргументами: {} {} {}.",
            child.id(),
            proxy_arg,
            bypass_arg,
            quic_arg
        )),
        Err(e) => Err(format!("Ошибка запуска Antigravity: {}", e)),
    }
}

pub fn patch_antigravity_desktop_shortcut(proxy_url: &str) -> Result<String, String> {
    let script = format!(
        r#"
        $w = New-Object -ComObject WScript.Shell
        $desktop = [System.Environment]::GetFolderPath('Desktop')
        $lnk = Join-Path $desktop 'Antigravity.lnk'
        if (-not (Test-Path $lnk)) {{
            $lnk = Join-Path $desktop 'Antigravity (Proxy).lnk'
        }}
        $target = "$env:LOCALAPPDATA\Programs\antigravity\Antigravity.exe"
        if (-not (Test-Path $target)) {{
            $target = "$env:PROGRAMFILES\antigravity\Antigravity.exe"
        }}
        $sc = $w.CreateShortcut($lnk)
        $sc.TargetPath = $target
        $sc.Arguments = '--proxy-server="{}" --proxy-bypass-list="localhost;127.0.0.1;[::1]" --disable-quic'
        $sc.WorkingDirectory = Split-Path $target
        $sc.IconLocation = "$target,0"
        $sc.Save()
        Write-Output "OK: $lnk"
        "#,
        proxy_url
    );

    let output = crate::diag::silent_command("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(format!(
            "Ярлык Antigravity на Рабочем столе обновлен. Теперь запуск по ярлыку всегда использует прокси {}.",
            proxy_url
        ))
    } else {
        Err(format!(
            "Ошибка создания ярлыка: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn open_antigravity_oauth_browser(proxy_url: &str) -> Result<String, String> {
    let oauth_url = match get_antigravity_log_oauth_url() {
        Some(u) => u,
        None => {
            return Err("Ссылка для входа пока не найдена в логах.\n\nШаги:\n1. Запустите Antigravity через кнопку выше\n2. Нажмите 'Sign In' в интерфейсе Antigravity\n3. Вернитесь сюда и нажмите эту кнопку — откроется браузер через европейский прокси.".to_string());
        }
    };

    // Find browser executable
    let browsers = [
        "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
        "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
        "C:\\Program Files\\BraveSoftware\\Brave-Browser\\Application\\brave.exe",
    ];

    let mut browser_exe = None;
    for b in browsers {
        if PathBuf::from(b).exists() {
            browser_exe = Some(b.to_string());
            break;
        }
    }

    if browser_exe.is_none() {
        if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
            let user_chrome = PathBuf::from(&local_app).join("Google\\Chrome\\Application\\chrome.exe");
            if user_chrome.exists() {
                browser_exe = Some(user_chrome.to_string_lossy().to_string());
            }
        }
    }

    let exe = match browser_exe {
        Some(e) => e,
        None => return Err("Не найден поддерживаемый браузер (Google Chrome, Microsoft Edge или Brave).".to_string()),
    };

    let user_data_dir = match std::env::var("LOCALAPPDATA") {
        Ok(l) => PathBuf::from(l).join("AI Access Doctor").join("oauth_browser_profile"),
        Err(_) => PathBuf::from("C:\\Temp\\AI_Access_Doctor_OAuth"),
    };
    let _ = std::fs::create_dir_all(&user_data_dir);

    let proxy_arg = format!("--proxy-server={}", proxy_url);
    let bypass_arg = "--proxy-bypass-list=localhost;127.0.0.1;[::1]";
    let quic_arg = "--disable-quic";
    let profile_arg = format!("--user-data-dir={}", user_data_dir.to_string_lossy());

    let mut cmd = std::process::Command::new(exe);
    cmd.args([
        &proxy_arg,
        bypass_arg,
        quic_arg,
        &profile_arg,
        "--no-first-run",
        "--no-default-browser-check",
        "--new-window",
        &oauth_url,
    ]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }

    match cmd.spawn() {
        Ok(_) => Ok("Изолированное окно браузера с европейским прокси запущено. Авторизуйтесь в Google, и токен автоматически передастся в Antigravity.".to_string()),
        Err(e) => Err(format!("Ошибка запуска браузера: {}", e)),
    }
}

