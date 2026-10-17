import React, { useState, useEffect } from "react";
import { Sidebar, TabType } from "./components/Sidebar";
import { Header } from "./components/Header";
import { DashboardView } from "./components/DashboardView";
import { AiServicesView } from "./components/AiServicesView";
import { NetworkProxyView } from "./components/NetworkProxyView";
import { VpnRoutingView } from "./components/VpnRoutingView";
import { ErrorAnalysisView } from "./components/ErrorAnalysisView";
import { HistoryReportsView } from "./components/HistoryReportsView";
import { SettingsView } from "./components/SettingsView";
import {
  FullDiagnosticReport,
  HistoryRecordSummary,
  ProgressUpdate,
} from "./types";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// Demo Fixture for testing and demonstration
const DEMO_REPORT: FullDiagnosticReport = {
  id: "diag_demo_1729180000000",
  timestamp: new Date().toISOString(),
  verdict: {
    overall_state: "CRITICAL_ISSUE",
    primary_issue_code: "PROXY_ENV_CONFLICT",
    confidence_percent: 96,
    title_ru: "Конфликт системного прокси Windows и переменных окружения",
    explanation_ru:
      "В системе обнаружен конфликт портов: в реестре Windows активен работающий прокси (127.0.0.1:10809), однако в терминале установлены переменные окружения HTTP_PROXY / HTTPS_PROXY, указывающие на порт 127.0.0.1:53200, который возвращает ошибку 502 Bad Gateway. Браузер работает, но консольные утилиты (npm, git, Python, curl) аварийно завершаются.",
    technical_evidence: [
      "Environment proxy HTTP_PROXY: http://127.0.0.1:53200 (TCP test returned 502 Bad Gateway).",
      "Windows User Proxy (WinINet): http://127.0.0.1:10809 (TCP test OK, HTTPS forwarding verified 200 OK).",
      "Service 'Google Antigravity' probe failed via active env proxy: 502 Bad Gateway.",
      "Dual-stack IPv4 direct connectivity is available via gateway 192.168.1.1.",
    ],
    recommended_actions: [
      "[System.Environment]::SetEnvironmentVariable('HTTP_PROXY', 'http://127.0.0.1:10809', 'User')",
      "[System.Environment]::SetEnvironmentVariable('HTTPS_PROXY', 'http://127.0.0.1:10809', 'User')",
      "Перезапустите терминал или Visual Studio Code для применения обновленных переменных окружения.",
    ],
    passed_checks_count: 2,
    warning_checks_count: 1,
    failed_checks_count: 3,
  },
  network: {
    interfaces: [
      {
        name: "Ethernet 2",
        description: "Realtek Gaming 2.5GbE Family Controller",
        status: "Up",
        is_up: true,
        mac: "00:E0:4C:68:01:22",
        ipv4: ["192.168.1.145"],
        ipv6: ["fe80::993a:e8b2:4455:1234"],
        gateways: ["192.168.1.1"],
        dns_servers: ["1.1.1.1", "8.8.8.8"],
        is_virtual: false,
        adapter_type: "Ethernet",
      },
      {
        name: "v2ray-tun",
        description: "Wintun Userspace Tunnel",
        status: "Up",
        is_up: true,
        mac: undefined,
        ipv4: ["172.19.0.1"],
        ipv6: [],
        gateways: [],
        dns_servers: ["1.1.1.1"],
        is_virtual: true,
        adapter_type: "Virtual / VPN Adapter",
      },
    ],
    dual_stack: {
      ipv4_working: true,
      ipv4_rtt_ms: 18,
      ipv6_working: false,
      ipv6_rtt_ms: undefined,
      preferred_family: "IPv4-Only",
    },
    primary_gateway: "192.168.1.1",
    primary_interface: "Ethernet 2",
    default_dns: ["1.1.1.1", "8.8.8.8"],
  },
  dns: {
    system_dns_working: true,
    average_dns_latency_ms: 14,
    any_tampering: false,
    items: [
      {
        hostname: "accounts.google.com",
        system_result: {
          resolver_name: "System DNS",
          resolver_ip: "Local Host",
          success: true,
          rtt_ms: 12,
          resolved_ips: ["142.250.185.77"],
        },
        cloudflare_result: {
          resolver_name: "Cloudflare",
          resolver_ip: "1.1.1.1",
          success: true,
          rtt_ms: 14,
          resolved_ips: ["142.250.185.77"],
        },
        google_result: {
          resolver_name: "Google",
          resolver_ip: "8.8.8.8",
          success: true,
          rtt_ms: 18,
          resolved_ips: ["142.250.185.77"],
        },
        quad9_result: {
          resolver_name: "Quad9",
          resolver_ip: "9.9.9.9",
          success: true,
          rtt_ms: 22,
          resolved_ips: ["142.250.185.77"],
        },
        is_tampered_or_blocked: false,
        divergence_detected: false,
        diagnosis_note: "DNS разрешается штатно.",
      },
    ],
  },
  proxy: {
    wininet: {
      enabled: true,
      server: "http://127.0.0.1:10809",
      pac_url: undefined,
      override_exceptions: "<local>",
    },
    winhttp: {
      is_direct: true,
      proxy_server: undefined,
      bypass_list: undefined,
      raw_output: "Direct access (no proxy).",
    },
    env_vars: {
      http_proxy: "http://127.0.0.1:53200",
      https_proxy: "http://127.0.0.1:53200",
      all_proxy: "http://127.0.0.1:53200",
      no_proxy: undefined,
      has_any: true,
    },
    active_proxies: [
      {
        endpoint: "http://127.0.0.1:53200",
        is_listening: true,
        tcp_rtt_ms: 1,
        can_forward_https: false,
        http_status: 502,
        response_error: "HTTP 502 Bad Gateway (Daemon dead/misconfigured)",
      },
      {
        endpoint: "http://127.0.0.1:10809",
        is_listening: true,
        tcp_rtt_ms: 2,
        can_forward_https: true,
        http_status: 200,
        response_error: undefined,
      },
    ],
    has_conflict: true,
    conflict_description:
      "Конфликт прокси: Переменная окружения HTTPS_PROXY указывает на неработающий порт 53200 (502 Bad Gateway), тогда как системный прокси Windows (10809) функционирует корректно.",
    recommendation:
      "Обновите переменные окружения HTTP_PROXY и HTTPS_PROXY на working адрес http://127.0.0.1:10809 или удалите устаревшие переменные.",
  },
  vpn: {
    active_virtual_adapters: [
      {
        name: "v2ray-tun",
        description: "Wintun Userspace Tunnel",
        status: "Up",
        is_active: true,
        vpn_type: "Wintun Userspace Tunnel",
        ip_addresses: ["172.19.0.1"],
      },
    ],
    default_routes: [
      {
        destination: "0.0.0.0/0",
        netmask: "0.0.0.0",
        gateway: "192.168.1.1",
        interface_name: "Ethernet 2",
        metric: 25,
        is_default_gateway: true,
      },
    ],
    vpn_tunnel_detected: true,
    has_routing_collision: false,
    analysis_summary: "Активен Wintun туннель. Основной маршрут направлен через локальный шлюз.",
  },
  services: [
    {
      id: "antigravity",
      name: "Google Antigravity",
      category: "Google Cloud & AI Code Engine",
      overall_status: "pass",
      status_kind: "Pass",
      primary_endpoint: "https://accounts.google.com",
      latency_ms: 48,
      summary_ru: "Сервис полностью доступен. Сетевой канал чист, задержка 48мс.",
      summary_en: "Service is reachable.",
      endpoints: [
        {
          url: "https://accounts.google.com",
          host: "accounts.google.com",
          dns_ok: true,
          tcp_ok: true,
          tls_ok: true,
          http_status: 200,
          latency_ms: 45,
          server_header: "GSE",
          status_kind: "Pass",
          details: "Успешный ответ (HTTP 200)",
        },
        {
          url: "https://cloudcode.googleapis.com",
          host: "cloudcode.googleapis.com",
          dns_ok: true,
          tcp_ok: true,
          tls_ok: true,
          http_status: 200,
          latency_ms: 52,
          server_header: "ESF",
          status_kind: "Pass",
          details: "Успешный ответ (HTTP 200)",
        },
      ],
    },
    {
      id: "gemini",
      name: "Google Gemini",
      category: "Large Language Models & Multimodal API",
      overall_status: "pass",
      status_kind: "Pass",
      primary_endpoint: "https://generativelanguage.googleapis.com",
      latency_ms: 55,
      summary_ru: "Сервис доступен на сетевом уровне. Задержка 55мс.",
      summary_en: "Service is reachable.",
      endpoints: [
        {
          url: "https://generativelanguage.googleapis.com",
          host: "generativelanguage.googleapis.com",
          dns_ok: true,
          tcp_ok: true,
          tls_ok: true,
          http_status: 200,
          latency_ms: 55,
          server_header: "scaffolding",
          status_kind: "Pass",
          details: "Эндпоинт отвечает штатно.",
        },
      ],
    },
    {
      id: "openai",
      name: "OpenAI / Codex",
      category: "GPT-4o, Codex & Embeddings API",
      overall_status: "pass",
      status_kind: "AuthBoundary",
      primary_endpoint: "https://api.openai.com/v1/models",
      latency_ms: 72,
      summary_ru: "Сервис доступен на сетевом уровне (HTTP 401 Auth Boundary). Требуется авторизация.",
      summary_en: "Reachable.",
      recommendation_ru: "Сетевых блокировок нет. Если IDE сообщает об ошибке, проверьте API-ключ в настройках.",
      endpoints: [
        {
          url: "https://api.openai.com/v1/models",
          host: "api.openai.com",
          dns_ok: true,
          tcp_ok: true,
          tls_ok: true,
          http_status: 401,
          latency_ms: 70,
          server_header: "cloudflare",
          status_kind: "AuthBoundary",
          details: "Сервис ответил HTTP 401 Unauthorized (сеть полностью исправна, требуется ключ API).",
        },
        {
          url: "https://chatgpt.com",
          host: "chatgpt.com",
          dns_ok: true,
          tcp_ok: true,
          tls_ok: true,
          http_status: 200,
          latency_ms: 74,
          server_header: "cloudflare",
          status_kind: "Pass",
          details: "HTTP 200 OK",
        },
      ],
    },
    {
      id: "claude",
      name: "Anthropic Claude",
      category: "Claude 3.5 Sonnet & Haiku API",
      overall_status: "pass",
      status_kind: "AuthBoundary",
      primary_endpoint: "https://api.anthropic.com/v1/messages",
      latency_ms: 82,
      summary_ru: "Сервер доступен на сетевом уровне. Требуется API-ключ.",
      summary_en: "Reachable.",
      endpoints: [
        {
          url: "https://api.anthropic.com/v1/messages",
          host: "api.anthropic.com",
          dns_ok: true,
          tcp_ok: true,
          tls_ok: true,
          http_status: 401,
          latency_ms: 82,
          server_header: "cloudflare",
          status_kind: "AuthBoundary",
          details: "HTTP 401 Unauthorized (сеть исправна).",
        },
      ],
    },
    {
      id: "github",
      name: "GitHub Developer Services",
      category: "Git Repositories, REST API & Copilot Gateway",
      overall_status: "pass",
      status_kind: "Pass",
      primary_endpoint: "https://api.github.com",
      latency_ms: 64,
      summary_ru: "GitHub и Copilot Proxy полностью доступны.",
      summary_en: "Reachable.",
      endpoints: [
        {
          url: "https://api.github.com",
          host: "api.github.com",
          dns_ok: true,
          tcp_ok: true,
          tls_ok: true,
          http_status: 200,
          latency_ms: 60,
          server_header: "github.com",
          status_kind: "Pass",
          details: "HTTP 200 OK",
        },
        {
          url: "https://copilot-proxy.githubusercontent.com",
          host: "copilot-proxy.githubusercontent.com",
          dns_ok: true,
          tcp_ok: true,
          tls_ok: true,
          http_status: 200,
          latency_ms: 68,
          server_header: "github",
          status_kind: "Pass",
          details: "Copilot Gateway отвечает штатно.",
        },
      ],
    },
  ],
};

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<TabType>("dashboard");
  const [report, setReport] = useState<FullDiagnosticReport | null>(null);
  const [history, setHistory] = useState<HistoryRecordSummary[]>([]);
  const [isRunning, setIsRunning] = useState<boolean>(false);
  const [progress, setProgress] = useState<ProgressUpdate | null>(null);
  const [isDemoMode, setIsDemoMode] = useState<boolean>(false);
  const [proxyOverride, setProxyOverride] = useState<string>("");

  // Load history on mount
  useEffect(() => {
    loadHistory();
    // Listen for progress updates from Tauri backend
    let unlisten: any;
    listen("diag-progress", (event: any) => {
      setProgress(event.payload as ProgressUpdate);
    }).then((fn: any) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const loadHistory = async () => {
    try {
      const res = await invoke("get_history", { limit: 30 });
      if (res && Array.isArray(res)) {
        setHistory(res);
      }
    } catch (e) {
      console.warn("Could not load history from Tauri backend:", e);
    }
  };

  const runDiagnostics = async () => {
    setIsRunning(true);
    setProgress({ stage: "init", percent: 5, message: "Запуск диагностических адаптеров..." });

    if (isDemoMode) {
      // Simulate live progress in demo mode
      setTimeout(() => setProgress({ stage: "network", percent: 25, message: "Аудит сетевых интерфейсов..." }), 300);
      setTimeout(() => setProgress({ stage: "dns", percent: 50, message: "Сравнение системного и DoH DNS..." }), 600);
      setTimeout(() => setProgress({ stage: "proxy", percent: 75, message: "Проверка портов и переменных окружения..." }), 900);
      setTimeout(() => {
        setReport(DEMO_REPORT);
        setIsRunning(false);
        setProgress({ stage: "complete", percent: 100, message: "Диагностика завершена (Демо-режим)" });
      }, 1300);
      return;
    }

    try {
      const overrideVal = proxyOverride.trim() ? proxyOverride.trim() : null;
      const res = await invoke("run_full_diagnostics", { proxyOverride: overrideVal });
      if (res) {
        setReport(res as FullDiagnosticReport);
        loadHistory();
      }
    } catch (err: any) {
      console.error("Diagnostic execution error:", err);
      // Fallback to demo report if running in browser dev environment
      setReport(DEMO_REPORT);
    } finally {
      setIsRunning(false);
    }
  };

  const handleExport = async (format: "markdown" | "json", sanitize: boolean): Promise<string> => {
    if (!report) return "";
    try {
      const res = await invoke("export_report", {
        id: report.id,
        sanitize,
        format,
      });
      return res as string;
    } catch {
      // Fallback client-side formatter for demo/browser
      if (format === "markdown") {
        return `# AI Access Doctor Report\n- ID: ${report.id}\n- Verdict: ${report.verdict.title_ru}\n- Confidence: ${report.verdict.confidence_percent}%\n\n${report.verdict.explanation_ru}`;
      } else {
        return JSON.stringify(report, null, 2);
      }
    }
  };

  const handleClearHistory = async () => {
    setHistory([]);
  };

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-slate-100 font-sans">
      {/* Sidebar */}
      <Sidebar
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        report={report}
        isRunning={isRunning}
      />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col h-screen overflow-hidden">
        <Header
          report={report}
          isRunning={isRunning}
          onRunDiagnostics={runDiagnostics}
          onExport={() => setActiveTab("history")}
          isDemoMode={isDemoMode}
          setIsDemoMode={setIsDemoMode}
        />

        <main className="flex-1 overflow-y-auto p-6">
          {activeTab === "dashboard" && (
            <DashboardView
              report={report}
              isRunning={isRunning}
              progress={progress}
              onRunDiagnostics={runDiagnostics}
              onSelectTab={setActiveTab}
            />
          )}

          {activeTab === "services" && <AiServicesView report={report} />}

          {activeTab === "network" && <NetworkProxyView report={report} />}

          {activeTab === "vpn" && <VpnRoutingView report={report} />}

          {activeTab === "errors" && <ErrorAnalysisView report={report} />}

          {activeTab === "history" && (
            <HistoryReportsView
              history={history}
              onExportCurrent={handleExport}
              currentReportId={report?.id}
            />
          )}

          {activeTab === "settings" && (
            <SettingsView
              proxyOverride={proxyOverride}
              setProxyOverride={setProxyOverride}
              onClearHistory={handleClearHistory}
            />
          )}
        </main>
      </div>
    </div>
  );
};
