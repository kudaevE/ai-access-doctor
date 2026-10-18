pub mod diag;
pub mod engine;
pub mod services;
pub mod storage;

use chrono::Utc;
use diag::comparison::{run_connection_matrix, ConnectionMatrixReport};
use diag::dns::{diagnose_dns_targets, DnsDiagnostics};
use diag::ide::{diagnose_ide_configs, IdeConfigAudit};
use diag::mcp::{diagnose_mcp_configs, McpFileAudit};
use diag::network::{diagnose_network, NetworkDiagnostics};
use diag::proxy::{diagnose_proxies, ProxyDiagnostics};
use diag::vpn::{diagnose_vpn, VpnDiagnostics};
use engine::detective::{analyze_error_text, ErrorDetectiveResult};
use engine::fix::{execute_safe_fix, get_available_fixes, FixExecutionResult, FixPreview};
use engine::rules::{evaluate_root_cause, DiagnosticVerdict};
use serde::{Deserialize, Serialize};
use services::{
    diagnose_antigravity, diagnose_claude, diagnose_gemini, diagnose_github, diagnose_openai,
    ServiceProfileResult,
};
use std::sync::Mutex;
use storage::db::{sanitize_text, DiagnosticStorage, HistoryRecordSummary};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullDiagnosticReport {
    pub id: String,
    pub timestamp: String,
    pub verdict: DiagnosticVerdict,
    pub network: NetworkDiagnostics,
    pub dns: DnsDiagnostics,
    pub proxy: ProxyDiagnostics,
    pub vpn: VpnDiagnostics,
    pub services: Vec<ServiceProfileResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub stage: String,
    pub percent: u32,
    pub message: String,
}

pub struct AppState {
    pub storage: Mutex<DiagnosticStorage>,
}

pub mod commands {
    use super::*;

    #[tauri::command]
    pub async fn run_full_diagnostics(
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        proxy_override: Option<String>,
    ) -> Result<FullDiagnosticReport, String> {
        let run_id = format!("diag_{}", Utc::now().timestamp_millis());

        let emit_progress = |stage: &str, percent: u32, msg: &str| {
            let _ = app.emit(
                "diag-progress",
                ProgressUpdate {
                    stage: stage.to_string(),
                    percent,
                    message: msg.to_string(),
                },
            );
        };

        // Stage 1: Network & Adapters
        emit_progress("network", 10, "Аудит сетевых интерфейсов и маршрутов...");
        let network_diag = diagnose_network().await;

        // Stage 2: DNS & Resolver Comparison
        emit_progress("dns", 25, "Сравнение системного и публичных DNS-резолверов...");
        let dns_targets = [
            "accounts.google.com",
            "generativelanguage.googleapis.com",
            "api.openai.com",
            "api.anthropic.com",
            "api.github.com",
        ];
        let dns_diag = diagnose_dns_targets(&dns_targets).await;

        // Stage 3: Proxy Audit
        emit_progress("proxy", 40, "Проверка WinINet, WinHTTP и переменных окружения...");
        let proxy_diag = diagnose_proxies().await;

        // Determine effective proxy to test with AI services if not explicitly overridden
        let effective_proxy = if let Some(p) = &proxy_override {
            Some(p.clone())
        } else {
            // If system WinINet proxy is working, or env proxy is working, pass it
            proxy_diag
                .active_proxies
                .iter()
                .find(|p| p.can_forward_https)
                .map(|p| p.endpoint.clone())
        };

        let proxy_ref = effective_proxy.as_deref();

        // Stage 4: VPN & Virtual Tunnels
        emit_progress("vpn", 55, "Анализ виртуальных VPN-адаптеров и приоритетов...");
        let vpn_diag = diagnose_vpn();

        // Stage 5: AI Service Endpoints
        emit_progress("services", 65, "Диагностика Google Antigravity & Cloud...");
        let antigravity_res = diagnose_antigravity(proxy_ref).await;

        emit_progress("services", 75, "Диагностика Google Gemini API...");
        let gemini_res = diagnose_gemini(proxy_ref).await;

        emit_progress("services", 82, "Диагностика OpenAI / Codex API...");
        let openai_res = diagnose_openai(proxy_ref).await;

        emit_progress("services", 89, "Диагностика Anthropic Claude API...");
        let claude_res = diagnose_claude(proxy_ref).await;

        emit_progress("services", 94, "Диагностика GitHub Developer Services...");
        let github_res = diagnose_github(proxy_ref).await;

        let services = vec![
            antigravity_res,
            gemini_res,
            openai_res,
            claude_res,
            github_res,
        ];

        // Stage 6: Rule Engine Inference
        emit_progress("inference", 98, "Анализ корневых причин и формирование рекомендаций...");
        let verdict = evaluate_root_cause(
            &network_diag,
            &dns_diag,
            &proxy_diag,
            &vpn_diag,
            &services,
        );

        let report = FullDiagnosticReport {
            id: run_id.clone(),
            timestamp: Utc::now().to_rfc3339(),
            verdict: verdict.clone(),
            network: network_diag,
            dns: dns_diag,
            proxy: proxy_diag,
            vpn: vpn_diag,
            services,
        };

        // Save to SQLite
        if let Ok(raw_json) = serde_json::to_string(&report) {
            if let Ok(storage) = state.storage.lock() {
                let _ = storage.save_run(
                    &run_id,
                    &verdict.overall_state,
                    &verdict.primary_issue_code,
                    &verdict.title_ru,
                    &verdict.explanation_ru,
                    verdict.passed_checks_count,
                    verdict.warning_checks_count,
                    verdict.failed_checks_count,
                    &raw_json,
                );
            }
        }

        emit_progress("complete", 100, "Диагностика успешно завершена!");

        Ok(report)
    }

    #[tauri::command]
    pub async fn get_network_info() -> Result<NetworkDiagnostics, String> {
        Ok(diagnose_network().await)
    }

    #[tauri::command]
    pub async fn get_proxy_info() -> Result<ProxyDiagnostics, String> {
        Ok(diagnose_proxies().await)
    }

    #[tauri::command]
    pub async fn get_vpn_info() -> Result<VpnDiagnostics, String> {
        Ok(diagnose_vpn())
    }

    #[tauri::command]
    pub async fn get_history(
        state: tauri::State<'_, AppState>,
        limit: Option<usize>,
    ) -> Result<Vec<HistoryRecordSummary>, String> {
        let lim = limit.unwrap_or(30);
        let storage = state.storage.lock().map_err(|e| e.to_string())?;
        storage.get_history(lim).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub async fn get_report_by_id(
        state: tauri::State<'_, AppState>,
        id: String,
    ) -> Result<Option<FullDiagnosticReport>, String> {
        let storage = state.storage.lock().map_err(|e| e.to_string())?;
        let json_opt = storage.get_run_json(&id).map_err(|e| e.to_string())?;
        if let Some(json_str) = json_opt {
            let rep: FullDiagnosticReport =
                serde_json::from_str(&json_str).map_err(|e| e.to_string())?;
            Ok(Some(rep))
        } else {
            Ok(None)
        }
    }

    #[tauri::command]
    pub async fn export_report(
        state: tauri::State<'_, AppState>,
        id: String,
        sanitize: bool,
        format: String,
    ) -> Result<String, String> {
        let storage = state.storage.lock().map_err(|e| e.to_string())?;
        let json_opt = storage.get_run_json(&id).map_err(|e| e.to_string())?;

        let json_str = match json_opt {
            Some(s) => s,
            None => return Err(format!("Report with ID {} not found", id)),
        };

        let report: FullDiagnosticReport =
            serde_json::from_str(&json_str).map_err(|e| e.to_string())?;

        if format.to_lowercase() == "markdown" {
            let mut md = format!(
                "# Отчет о диагностике сетевого доступа AI Access Doctor\n\n- **Дата:** {}\n- **ID запуска:** {}\n- **Вердикт:** {} ({})\n- **Достоверность:** {}%\n\n",
                report.timestamp, report.id, report.verdict.title_ru, report.verdict.overall_state, report.verdict.confidence_percent
            );

            md.push_str("## Корневая причина и объяснение\n\n");
            md.push_str(&format!("{}\n\n", report.verdict.explanation_ru));

            md.push_str("## Рекомендованные действия\n\n");
            for act in &report.verdict.recommended_actions {
                md.push_str(&format!("- {}\n", act));
            }

            md.push_str("\n## Статус ИИ-сервисов\n\n| Сервис | Статус | Задержка | Первичный эндпоинт |\n|---|---|---|---|\n");
            for s in &report.services {
                md.push_str(&format!(
                    "| {} | {} | {}ms | {} |\n",
                    s.name, s.overall_status.to_uppercase(), s.latency_ms, s.primary_endpoint
                ));
            }

            md.push_str("\n## Технические свидетельства\n\n```text\n");
            for ev in &report.verdict.technical_evidence {
                md.push_str(&format!("{}\n", ev));
            }
            md.push_str("```\n");

            if sanitize {
                Ok(sanitize_text(&md))
            } else {
                Ok(md)
            }
        } else {
            // JSON format
            if sanitize {
                Ok(sanitize_text(&json_str))
            } else {
                Ok(json_str)
            }
        }
    }

    #[tauri::command]
    pub async fn analyze_error_log(error_text: String) -> Result<ErrorDetectiveResult, String> {
        Ok(analyze_error_text(&error_text))
    }

    #[tauri::command]
    pub async fn compare_connection_routes(
        target_url: Option<String>,
    ) -> Result<ConnectionMatrixReport, String> {
        Ok(run_connection_matrix(target_url.as_deref()).await)
    }

    #[tauri::command]
    pub async fn get_ide_audits() -> Result<Vec<IdeConfigAudit>, String> {
        Ok(diagnose_ide_configs())
    }

    #[tauri::command]
    pub async fn get_mcp_audits() -> Result<Vec<McpFileAudit>, String> {
        Ok(diagnose_mcp_configs())
    }

    #[tauri::command]
    pub async fn get_safe_fixes() -> Result<Vec<FixPreview>, String> {
        Ok(get_available_fixes())
    }

    #[tauri::command]
    pub async fn apply_safe_fix(fix_id: String) -> Result<FixExecutionResult, String> {
        execute_safe_fix(&fix_id)
    }

    #[tauri::command]
    pub async fn get_free_hub() -> Result<diag::free_hub::FreeHubReport, String> {
        Ok(diag::free_hub::get_free_hub_data().await)
    }

    #[tauri::command]
    pub fn get_one_click_status() -> Result<diag::free_hub::OneClickStatus, String> {
        Ok(diag::free_hub::get_one_click_status_internal())
    }

    #[tauri::command]
    pub fn apply_one_click_preset(preset_id: String) -> Result<String, String> {
        diag::free_hub::apply_one_click_preset_internal(&preset_id)
    }

    #[tauri::command]
    pub fn rollback_all_settings() -> Result<String, String> {
        diag::free_hub::rollback_all_settings_internal()
    }

    #[tauri::command]
    pub async fn auto_troubleshoot_and_select() -> Result<diag::free_hub::AutoTroubleshootReport, String> {
        diag::free_hub::auto_troubleshoot_and_select().await
    }

    #[tauri::command]
    pub async fn get_antigravity_status() -> Result<diag::antigravity_unlock::AntigravityDoctorReport, String> {
        Ok(diag::antigravity_unlock::audit_antigravity_doctor().await)
    }

    #[tauri::command]
    pub fn apply_antigravity_unlock(profile: String) -> Result<String, String> {
        diag::antigravity_unlock::apply_antigravity_unlock_profile(&profile)
    }

    #[tauri::command]
    pub fn rollback_antigravity_unlock() -> Result<String, String> {
        diag::antigravity_unlock::rollback_antigravity_unlock()
    }

    #[tauri::command]
    pub fn get_google_flow_info(app: AppHandle) -> Result<diag::flow_unlocker::FlowUnlockInfo, String> {
        Ok(diag::flow_unlocker::get_flow_info(&app))
    }

    #[tauri::command]
    pub fn open_google_flow(app: AppHandle) -> Result<String, String> {
        diag::flow_unlocker::launch_google_flow_window(&app)
    }
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            storage: Mutex::new(DiagnosticStorage::new()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::run_full_diagnostics,
            commands::get_network_info,
            commands::get_proxy_info,
            commands::get_vpn_info,
            commands::get_history,
            commands::get_report_by_id,
            commands::export_report,
            commands::analyze_error_log,
            commands::compare_connection_routes,
            commands::get_ide_audits,
            commands::get_mcp_audits,
            commands::get_safe_fixes,
            commands::apply_safe_fix,
            commands::get_free_hub,
            commands::get_one_click_status,
            commands::apply_one_click_preset,
            commands::rollback_all_settings,
            commands::auto_troubleshoot_and_select,
            commands::get_antigravity_status,
            commands::apply_antigravity_unlock,
            commands::rollback_antigravity_unlock,
            commands::get_google_flow_info,
            commands::open_google_flow,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
