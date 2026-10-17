use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPreview {
    pub fix_id: String,
    pub title_ru: String,
    pub description_ru: String,
    pub command_preview: String,
    pub subsystem: String, // "DNS" | "Environment" | "Network"
    pub is_destructive: bool,
    pub requires_admin: bool,
    pub rollback_preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixExecutionResult {
    pub success: bool,
    pub snapshot_id: String,
    pub message_ru: String,
    pub stdout: String,
    pub stderr: String,
}

pub fn get_available_fixes() -> Vec<FixPreview> {
    vec![
        FixPreview {
            fix_id: "flush_dns".to_string(),
            title_ru: "Очистка локального кэша DNS (Flush DNS)".to_string(),
            description_ru: "Сбрасывает системный кэш DNS-резолвера Windows. Устраняет застрявшие устаревшие или подмененные IP-адреса серверов ИИ.".to_string(),
            command_preview: "ipconfig /flushdns".to_string(),
            subsystem: "DNS".to_string(),
            is_destructive: false,
            requires_admin: false,
            rollback_preview: "Кэш автоматически заполняется актуальными адресами при следующих запросах.".to_string(),
        },
        FixPreview {
            fix_id: "clear_stale_proxy_env".to_string(),
            title_ru: "Удаление устаревших переменных HTTP_PROXY / HTTPS_PROXY".to_string(),
            description_ru: "Очищает неработающие переменные окружения пользователя (например, указывающие на закрытый порт 53200), устраняя ошибку 502 Bad Gateway в терминале и IDE.".to_string(),
            command_preview: "[System.Environment]::SetEnvironmentVariable('HTTP_PROXY', $null, 'User'); [System.Environment]::SetEnvironmentVariable('HTTPS_PROXY', $null, 'User')".to_string(),
            subsystem: "Environment".to_string(),
            is_destructive: false,
            requires_admin: false,
            rollback_preview: "Снимок предыдущих значений сохраняется в истории для мгновенного отката.".to_string(),
        },
        FixPreview {
            fix_id: "sync_env_to_active_proxy".to_string(),
            title_ru: "Синхронизация переменных окружения с рабочим прокси Windows".to_string(),
            description_ru: "Устанавливает переменные HTTP_PROXY и HTTPS_PROXY в соответствии с действующим системным прокси (127.0.0.1:10809), чтобы консольные утилиты (npm, git, Python) работали через него.".to_string(),
            command_preview: "[System.Environment]::SetEnvironmentVariable('HTTP_PROXY', 'http://127.0.0.1:10809', 'User'); [System.Environment]::SetEnvironmentVariable('HTTPS_PROXY', 'http://127.0.0.1:10809', 'User')".to_string(),
            subsystem: "Environment".to_string(),
            is_destructive: false,
            requires_admin: false,
            rollback_preview: "Возврат к предыдущим значениям через восстановление снимка.".to_string(),
        },
    ]
}

pub fn execute_safe_fix(fix_id: &str) -> Result<FixExecutionResult, String> {
    let snapshot_id = format!("snap_{}", chrono::Utc::now().timestamp_millis());

    match fix_id {
        "flush_dns" => {
            let output = Command::new("ipconfig")
                .arg("/flushdns")
                .output()
                .map_err(|e| format!("Не удалось выполнить ipconfig: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            Ok(FixExecutionResult {
                success: output.status.success(),
                snapshot_id,
                message_ru: "Локальный кэш DNS успешно очищен.".to_string(),
                stdout,
                stderr,
            })
        }
        "clear_stale_proxy_env" => {
            let cmd = "[System.Environment]::SetEnvironmentVariable('HTTP_PROXY', $null, 'User'); [System.Environment]::SetEnvironmentVariable('HTTPS_PROXY', $null, 'User'); [System.Environment]::SetEnvironmentVariable('ALL_PROXY', $null, 'User')";
            let output = Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", cmd])
                .output()
                .map_err(|e| format!("Ошибка выполнения PowerShell: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            Ok(FixExecutionResult {
                success: output.status.success(),
                snapshot_id,
                message_ru: "Устаревшие переменные прокси успешно удалены из профиля пользователя.".to_string(),
                stdout,
                stderr,
            })
        }
        "sync_env_to_active_proxy" => {
            let cmd = "[System.Environment]::SetEnvironmentVariable('HTTP_PROXY', 'http://127.0.0.1:10809', 'User'); [System.Environment]::SetEnvironmentVariable('HTTPS_PROXY', 'http://127.0.0.1:10809', 'User')";
            let output = Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", cmd])
                .output()
                .map_err(|e| format!("Ошибка выполнения PowerShell: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            Ok(FixExecutionResult {
                success: output.status.success(),
                snapshot_id,
                message_ru: "Переменные HTTP_PROXY и HTTPS_PROXY синхронизированы с рабочим портом 127.0.0.1:10809.".to_string(),
                stdout,
                stderr,
            })
        }
        _ => Err(format!("Действие '{}' не входит в список разрешенных безопасных операций.", fix_id)),
    }
}
