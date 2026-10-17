use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertDetail {
    pub subject: String,
    pub issuer: String,
    pub not_before: String,
    pub not_after: String,
    pub is_self_signed: bool,
    pub is_mitm_or_inspection: bool,
    pub interceptor_detected: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsProbeResult {
    pub target: String,
    pub port: u16,
    pub handshake_success: bool,
    pub tls_version: Option<String>,
    pub cipher_suite: Option<String>,
    pub alpn_negotiated: Option<String>,
    pub rtt_ms: Option<u64>,
    pub cert_chain: Vec<CertDetail>,
    pub error: Option<String>,
    pub error_kind: Option<String>,
}

/// Analyze certificate for middlebox / MITM / Antivirus interception
fn inspect_cert(der: &[u8]) -> Option<CertDetail> {
    if let Ok((_, cert)) = x509_parser::parse_x509_certificate(der) {
        let subject = cert.subject().to_string();
        let issuer = cert.issuer().to_string();
        let not_before = cert.validity().not_before.to_string();
        let not_after = cert.validity().not_after.to_string();

        let is_self_signed = subject == issuer;

        let lower_issuer = issuer.to_lowercase();
        let mut interceptor = None;
        let known_interceptors = [
            ("kaspersky", "Kaspersky Anti-Virus HTTPS Filtering"),
            ("eset", "ESET SSL/TLS Inspection"),
            ("bitdefender", "Bitdefender Encrypted Web Scan"),
            ("avast", "Avast Web Shield"),
            ("zscaler", "Zscaler Corporate Inspection Proxy"),
            ("fortinet", "Fortinet FortiGate SSL Deep Inspection"),
            ("palo alto", "Palo Alto Networks Decryption"),
            ("sophos", "Sophos Web Protection"),
            ("adguard", "AdGuard Personal CA"),
            ("charles", "Charles Proxy CA"),
            ("fiddler", "Fiddler Root Certificate"),
            ("mitmproxy", "mitmproxy CA"),
        ];

        for (pattern, name) in &known_interceptors {
            if lower_issuer.contains(pattern) {
                interceptor = Some(name.to_string());
                break;
            }
        }

        let is_mitm_or_inspection = interceptor.is_some();

        Some(CertDetail {
            subject,
            issuer,
            not_before,
            not_after,
            is_self_signed,
            is_mitm_or_inspection,
            interceptor_detected: interceptor,
        })
    } else {
        None
    }
}

/// Run TLS handshake diagnostics against target host on specified port
pub async fn probe_tls(host: &str, port: u16) -> TlsProbeResult {
    let start = Instant::now();
    let addr_spec = format!("{}:{}", host, port);

    // 1. Establish TCP stream first
    let tcp_stream = match tokio::time::timeout(Duration::from_millis(3000), TcpStream::connect(&addr_spec)).await {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            return TlsProbeResult {
                target: host.to_string(),
                port,
                handshake_success: false,
                tls_version: None,
                cipher_suite: None,
                alpn_negotiated: None,
                rtt_ms: None,
                cert_chain: Vec::new(),
                error: Some(format!("TCP connection failed: {}", e)),
                error_kind: Some("TCP_CONNECT_FAIL".to_string()),
            };
        }
        Err(_) => {
            return TlsProbeResult {
                target: host.to_string(),
                port,
                handshake_success: false,
                tls_version: None,
                cipher_suite: None,
                alpn_negotiated: None,
                rtt_ms: None,
                cert_chain: Vec::new(),
                error: Some("TCP connection timed out".to_string()),
                error_kind: Some("TCP_TIMEOUT".to_string()),
            };
        }
    };

    // 2. Configure rustls client
    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let mut client_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    client_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    let connector = TlsConnector::from(Arc::new(client_config));

    let server_name = match ServerName::try_from(host.to_string()) {
        Ok(name) => name,
        Err(e) => {
            return TlsProbeResult {
                target: host.to_string(),
                port,
                handshake_success: false,
                tls_version: None,
                cipher_suite: None,
                alpn_negotiated: None,
                rtt_ms: None,
                cert_chain: Vec::new(),
                error: Some(format!("Invalid SNI server name: {}", e)),
                error_kind: Some("INVALID_SNI".to_string()),
            };
        }
    };

    // 3. Perform TLS Handshake
    match tokio::time::timeout(Duration::from_millis(4000), connector.connect(server_name, tcp_stream)).await {
        Ok(Ok(tls_stream)) => {
            let rtt = start.elapsed().as_millis() as u64;
            let (_, session) = tls_stream.into_inner();

            let tls_version = session.protocol_version().map(|v| match v {
                tokio_rustls::rustls::ProtocolVersion::TLSv1_2 => "TLSv1.2".to_string(),
                tokio_rustls::rustls::ProtocolVersion::TLSv1_3 => "TLSv1.3".to_string(),
                _ => format!("{:?}", v),
            });

            let cipher_suite = session.negotiated_cipher_suite().map(|c| format!("{:?}", c.suite()));

            let alpn_negotiated = session
                .alpn_protocol()
                .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok());

            let mut cert_chain = Vec::new();
            if let Some(peer_certs) = session.peer_certificates() {
                for cert in peer_certs {
                    if let Some(detail) = inspect_cert(cert.as_ref()) {
                        cert_chain.push(detail);
                    }
                }
            }

            TlsProbeResult {
                target: host.to_string(),
                port,
                handshake_success: true,
                tls_version,
                cipher_suite,
                alpn_negotiated,
                rtt_ms: Some(rtt),
                cert_chain,
                error: None,
                error_kind: None,
            }
        }
        Ok(Err(e)) => {
            let err_str = e.to_string();
            let error_kind = if err_str.contains("UnknownIssuer") || err_str.contains("BadSignature") {
                "CERTIFICATE_UNTRUSTED".to_string()
            } else if err_str.contains("CertExpired") {
                "CERTIFICATE_EXPIRED".to_string()
            } else if err_str.contains("CertNotValidForName") {
                "CERTIFICATE_HOSTNAME_MISMATCH".to_string()
            } else {
                "TLS_HANDSHAKE_ERROR".to_string()
            };

            TlsProbeResult {
                target: host.to_string(),
                port,
                handshake_success: false,
                tls_version: None,
                cipher_suite: None,
                alpn_negotiated: None,
                rtt_ms: Some(start.elapsed().as_millis() as u64),
                cert_chain: Vec::new(),
                error: Some(err_str),
                error_kind: Some(error_kind),
            }
        }
        Err(_) => TlsProbeResult {
            target: host.to_string(),
            port,
            handshake_success: false,
            tls_version: None,
            cipher_suite: None,
            alpn_negotiated: None,
            rtt_ms: None,
            cert_chain: Vec::new(),
            error: Some("TLS handshake timed out after 4000ms".to_string()),
            error_kind: Some("TLS_HANDSHAKE_TIMEOUT".to_string()),
        },
    }
}
