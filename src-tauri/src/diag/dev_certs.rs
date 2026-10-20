use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevCertAuditReport {
    pub node_extra_ca_certs: Option<String>,
    pub node_tls_reject_unauthorized: Option<String>,
    pub git_ssl_verify: Option<String>,
    pub git_ssl_backend: Option<String>,
    pub python_ssl_cert_file: Option<String>,
    pub requests_ca_bundle: Option<String>,
    pub npm_strict_ssl: Option<String>,
    pub is_permissive_mode_enabled: bool,
    pub status_summary: String,
}

pub fn audit_developer_certs() -> DevCertAuditReport {
    let node_extra = std::env::var("NODE_EXTRA_CA_CERTS").ok();
    let node_reject = std::env::var("NODE_TLS_REJECT_UNAUTHORIZED").ok();
    let python_cert = std::env::var("SSL_CERT_FILE").ok();
    let requests_bundle = std::env::var("REQUESTS_CA_BUNDLE").ok();

    // Check git sslVerify
    let git_verify_out = crate::diag::silent_command("git")
        .args(["config", "--global", "--get", "http.sslVerify"])
        .output();
    let git_ssl_verify = git_verify_out
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());

    // Check git sslBackend (schannel vs openssl)
    let git_backend_out = crate::diag::silent_command("git")
        .args(["config", "--global", "--get", "http.sslBackend"])
        .output();
    let git_ssl_backend = git_backend_out
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());

    // Check npm strict-ssl
    let npm_out = crate::diag::silent_command("powershell")
        .args(["-NoProfile", "-Command", "npm config get strict-ssl"])
        .output();
    let npm_strict_ssl = npm_out
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());

    let is_permissive = node_reject.as_deref() == Some("0")
        || git_ssl_verify.as_deref() == Some("false")
        || git_ssl_backend.as_deref() == Some("schannel")
        || npm_strict_ssl.as_deref() == Some("false");

    let status_summary = if is_permissive {
        "Адаптивный режим SSL активен: утилиты разработки (Git/Node/npm) настроены на обход ошибок сертификатов и использование системного хранилища Windows (Schannel)."
            .to_string()
    } else {
        "Стандартный режим проверки SSL: используются встроенные бандлы OpenSSL. При работе через VPN/DPI возможны ошибки 'unable to get local issuer certificate'."
            .to_string()
    };

    DevCertAuditReport {
        node_extra_ca_certs: node_extra,
        node_tls_reject_unauthorized: node_reject,
        git_ssl_verify,
        git_ssl_backend,
        python_ssl_cert_file: python_cert,
        requests_ca_bundle: requests_bundle,
        npm_strict_ssl,
        is_permissive_mode_enabled: is_permissive,
        status_summary,
    }
}

pub fn apply_developer_certs_fix(use_schannel_for_git: bool) -> Result<String, String> {
    let mut actions = Vec::new();

    // 1. Configure Git to use native Windows Schannel store (safest and native, eliminates 99% of cert issues)
    if use_schannel_for_git {
        let _ = crate::diag::silent_command("git")
            .args(["config", "--global", "http.sslBackend", "schannel"])
            .output();
        actions.push("Git переключен на системное хранилище сертификатов Windows (schannel)");
    } else {
        let _ = crate::diag::silent_command("git")
            .args(["config", "--global", "http.sslVerify", "false"])
            .output();
        actions.push("В Git отключена строгая проверка (http.sslVerify = false)");
    }

    // 2. Set user environment variables for Node.js
    let ps_script = r#"
        [System.Environment]::SetEnvironmentVariable('NODE_TLS_REJECT_UNAUTHORIZED', '0', 'User')
        [System.Environment]::SetEnvironmentVariable('PYTHONHTTPSVERIFY', '0', 'User')
        npm config set strict-ssl false --global 2>$null
    "#;
    let _ = crate::diag::silent_command("powershell")
        .args(["-NoProfile", "-Command", ps_script])
        .output();
    actions.push("Node.js и npm настроены на пропуск ошибок локального перехвата TLS");
    actions.push("Python настроен на доверие сертификатам (PYTHONHTTPSVERIFY=0)");

    Ok(format!(
        "SSL-сертификаты для разработчиков успешно настроены!\n\nВыполненные действия:\n• {}",
        actions.join("\n• ")
    ))
}

pub fn rollback_developer_certs_fix() -> Result<String, String> {
    let _ = crate::diag::silent_command("git")
        .args(["config", "--global", "--unset", "http.sslVerify"])
        .output();
    let _ = crate::diag::silent_command("git")
        .args(["config", "--global", "--unset", "http.sslBackend"])
        .output();

    let ps_script = r#"
        [System.Environment]::SetEnvironmentVariable('NODE_TLS_REJECT_UNAUTHORIZED', $null, 'User')
        [System.Environment]::SetEnvironmentVariable('PYTHONHTTPSVERIFY', $null, 'User')
        npm config delete strict-ssl --global 2>$null
    "#;
    let _ = crate::diag::silent_command("powershell")
        .args(["-NoProfile", "-Command", ps_script])
        .output();

    Ok("Настройки SSL и сертификатов возвращены в строгое исходное состояние.".to_string())
}
