use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostsEntryInfo {
    pub line_number: usize,
    pub ip: String,
    pub domain: String,
    pub is_ai_domain: bool,
    pub is_commented: bool,
    pub is_suspicious_or_dead: bool,
    pub note_ru: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostsDoctorReport {
    pub hosts_path: String,
    pub total_lines: usize,
    pub ai_entries_count: usize,
    pub suspicious_entries_count: usize,
    pub entries: Vec<HostsEntryInfo>,
    pub has_backup: bool,
    pub status_ru: String,
}

const AI_DOMAINS: &[&str] = &[
    "cloudcode-pa.googleapis.com",
    "daily-cloudcode-pa.googleapis.com",
    "accounts.google.com",
    "generativelanguage.googleapis.com",
    "flow.google.com",
    "googleapis.com",
    "api.openai.com",
    "api.anthropic.com",
    "chatgpt.com",
    "claude.ai",
    "api.github.com",
];

fn get_hosts_path() -> PathBuf {
    PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
}

pub fn audit_hosts_file() -> HostsDoctorReport {
    let hosts_path = get_hosts_path();
    let path_str = hosts_path.to_string_lossy().to_string();

    let backup_path = PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts.bak");
    let has_backup = backup_path.exists();

    let content = match fs::read_to_string(&hosts_path) {
        Ok(c) => c,
        Err(e) => {
            return HostsDoctorReport {
                hosts_path: path_str,
                total_lines: 0,
                ai_entries_count: 0,
                suspicious_entries_count: 0,
                entries: Vec::new(),
                has_backup,
                status_ru: format!("Не удалось прочитать файл hosts: {}", e),
            };
        }
    };

    let mut entries = Vec::new();
    let mut ai_count = 0;
    let mut suspicious_count = 0;
    let mut total_lines = 0;

    for (idx, line) in content.lines().enumerate() {
        total_lines += 1;
        let line_num = idx + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let is_commented = trimmed.starts_with('#');
        let clean_line = if is_commented {
            trimmed.trim_start_matches('#').trim()
        } else {
            trimmed
        };

        let parts: Vec<&str> = clean_line.split_whitespace().collect();
        if parts.len() >= 2 {
            let ip = parts[0];
            for domain in &parts[1..] {
                let domain_lower = domain.to_lowercase();
                let is_ai = AI_DOMAINS.iter().any(|d| domain_lower.contains(*d));

                if is_ai {
                    ai_count += 1;
                    let is_rkn = ip == "185.149.145.6";
                    let is_loopback = ip == "127.0.0.1" || ip == "0.0.0.0" || ip == "::1";
                    let is_suspicious = is_rkn || is_loopback || !is_commented;

                    if is_suspicious {
                        suspicious_count += 1;
                    }

                    let note_ru = if is_rkn {
                        "Внимание: адрес заглушки РКН (185.149.145.6). Доступ полностью блокируется!".to_string()
                    } else if is_loopback {
                        "Закольцован на локальный хост (127.0.0.1/0.0.0.0). Запросы не уйдут в сеть.".to_string()
                    } else if !is_commented {
                        "Статический IP в обход DNS. Если IP устареет, ИИ перестанет отвечать.".to_string()
                    } else {
                        "Закомментированная запись (не активна).".to_string()
                    };

                    entries.push(HostsEntryInfo {
                        line_number: line_num,
                        ip: ip.to_string(),
                        domain: domain.to_string(),
                        is_ai_domain: true,
                        is_commented,
                        is_suspicious_or_dead: is_suspicious,
                        note_ru,
                    });
                }
            }
        }
    }

    let status_ru = if suspicious_count > 0 {
        format!(
            "Обнаружено {} подозрительных записей для доменов ИИ в hosts! Рекомендуется очистить их.",
            suspicious_count
        )
    } else if ai_count > 0 {
        format!("Найдено {} записей ИИ в hosts (все безопасны или отключены).", ai_count)
    } else {
        "Файл hosts чист: никаких конфликтующих записей для ИИ-сервисов не обнаружено.".to_string()
    };

    HostsDoctorReport {
        hosts_path: path_str,
        total_lines,
        ai_entries_count: ai_count,
        suspicious_entries_count: suspicious_count,
        entries,
        has_backup,
        status_ru,
    }
}

pub fn clean_ai_hosts_entries() -> Result<String, String> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    use std::os::windows::process::CommandExt;

    let domains_pattern = AI_DOMAINS.join("|");
    let script = format!(
        r#"
        $hosts = "$env:windir\System32\drivers\etc\hosts"
        $backup = "$env:windir\System32\drivers\etc\hosts.bak"
        Copy-Item -Path $hosts -Destination $backup -Force
        $lines = Get-Content -Path $hosts
        $clean = $lines | Where-Object {{ $_ -notmatch '({})' }}
        Set-Content -Path $hosts -Value $clean -Force
        ipconfig /flushdns
        "#,
        domains_pattern
    );

    let elev_cmd = format!(
        "Start-Process powershell -Verb RunAs -Wait -WindowStyle Hidden -ArgumentList \"-NoProfile -Command {}\"",
        script.replace('"', "\\\"")
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &elev_cmd])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Не удалось запустить очистку hosts: {}", e))?;

    if output.status.success() {
        Ok("Файл hosts успешно очищен от записей ИИ! Создана копия hosts.bak, кэш DNS сброшен.".to_string())
    } else {
        Err(format!(
            "Ошибка очистки hosts: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn open_etc_directory() -> Result<String, String> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    use std::os::windows::process::CommandExt;

    Command::new("explorer")
        .arg(r"C:\Windows\System32\drivers\etc")
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("Не удалось открыть папку etc: {}", e))?;

    Ok("Папка C:\\Windows\\System32\\drivers\\etc открыта в Проводнике.".to_string())
}
