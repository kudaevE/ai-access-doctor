import React, { useState, useEffect } from "react";
import {
  Code2,
  Cpu,
  RotateCw,
  CheckCircle2,
  AlertTriangle,
  MinusCircle,
  FileCode2,
  Layers,
  Terminal,
  ExternalLink,
  ShieldCheck,
  Server,
  FolderOpen,
  Zap,
  Globe,
  Lock,
  Unlock,
  Check,
  Sparkles,
} from "lucide-react";
import { IdeConfigAudit, McpFileAudit, DevCertAuditReport } from "../types";
import { invoke } from "@tauri-apps/api/core";

export const IdeDoctorView: React.FC = () => {
  const [ideAudits, setIdeAudits] = useState<IdeConfigAudit[]>([]);
  const [mcpAudits, setMcpAudits] = useState<McpFileAudit[]>([]);
  const [devCerts, setDevCerts] = useState<DevCertAuditReport | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  const [activeSubTab, setActiveSubTab] = useState<"ide" | "mcp" | "ssl">("ide");
  const [notification, setNotification] = useState<{ message: string; type: "success" | "error" } | null>(null);

  const notify = (msg: string, type: "success" | "error" = "success") => {
    setNotification({ message: msg, type });
    setTimeout(() => setNotification(null), 4000);
  };

  const runAudit = async () => {
    setIsLoading(true);
    try {
      const [ides, mcps, certs] = await Promise.all([
        invoke<IdeConfigAudit[]>("get_ide_audits"),
        invoke<McpFileAudit[]>("get_mcp_audits"),
        invoke<DevCertAuditReport>("audit_developer_certs"),
      ]);
      if (ides && Array.isArray(ides)) setIdeAudits(ides);
      if (mcps && Array.isArray(mcps)) setMcpAudits(mcps);
      if (certs) setDevCerts(certs);
    } catch (e) {
      console.error("IDE/MCP Audit error:", e);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    runAudit();
  }, []);

  const handleFixIde = async (ideId: string) => {
    setActionLoading(`fix_${ideId}`);
    try {
      const res = await invoke<string>("fix_ide_proxy_and_ssl", {
        ideId,
        proxyUrl: "http://127.0.0.1:53200",
        disableSsl: true,
      });
      notify(res, "success");
      await runAudit();
    } catch (e: any) {
      notify(`Ошибка: ${e}`, "error");
    } finally {
      setActionLoading(null);
    }
  };

  const handleResetIde = async (ideId: string) => {
    setActionLoading(`reset_${ideId}`);
    try {
      const res = await invoke<string>("reset_ide_proxy", { ideId });
      notify(res, "success");
      await runAudit();
    } catch (e: any) {
      notify(`Ошибка: ${e}`, "error");
    } finally {
      setActionLoading(null);
    }
  };

  const handleApplyDevCerts = async () => {
    setActionLoading("dev_certs_apply");
    try {
      const res = await invoke<string>("apply_developer_certs_fix", { useSchannelForGit: true });
      notify(res, "success");
      await runAudit();
    } catch (e: any) {
      notify(`Ошибка: ${e}`, "error");
    } finally {
      setActionLoading(null);
    }
  };

  const handleRollbackDevCerts = async () => {
    setActionLoading("dev_certs_rollback");
    try {
      const res = await invoke<string>("rollback_developer_certs_fix");
      notify(res, "success");
      await runAudit();
    } catch (e: any) {
      notify(`Ошибка: ${e}`, "error");
    } finally {
      setActionLoading(null);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="rounded-2xl bg-surface border border-surface-border p-6 shadow-xl relative overflow-hidden">
        <div className="absolute top-0 right-0 -mt-10 -mr-10 w-64 h-64 bg-primary/10 rounded-full blur-3xl pointer-events-none" />

        <div className="relative z-10 flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <div className="flex items-center gap-2 mb-1.5">
              <span className="text-[11px] font-semibold uppercase tracking-wider px-2 py-0.5 rounded bg-primary/15 text-primary border border-primary/25">
                AI Ecosystem & DevTools Doctor
              </span>
              <span className="text-xs text-slate-400 font-mono">
                Аудит редакторов кода, MCP-серверов и сертификатов SSL
              </span>
            </div>
            <h2 className="text-xl font-bold text-white tracking-tight">
              Инспекция Cursor, Windsurf, VS Code, MCP и Dev SSL
            </h2>
            <p className="text-xs text-slate-400 max-w-2xl mt-1">
              Централизованное управление прокси-настройками для Cursor, Windsurf, Claude Desktop,
              а также 1-клик устранение ошибок сертификатов в Git, Node.js и Python.
            </p>
          </div>

          <button
            onClick={runAudit}
            disabled={isLoading}
            className={`flex items-center gap-2.5 px-5 py-2.5 rounded-xl font-semibold text-xs shadow-lg transition-all active:scale-95 ${
              isLoading
                ? "bg-slate-800 text-slate-500 cursor-not-allowed border border-surface-border"
                : "bg-primary text-slate-950 hover:bg-primary-hover shadow-primary/20"
            }`}
          >
            <RotateCw className={`w-4 h-4 ${isLoading ? "animate-spin" : ""}`} />
            <span>{isLoading ? "Сканирование..." : "Пересканировать"}</span>
          </button>
        </div>

        {/* Sub-tab Switcher */}
        <div className="flex flex-wrap gap-2 mt-5 pt-4 border-t border-surface-border/60">
          <button
            onClick={() => setActiveSubTab("ide")}
            className={`flex items-center gap-2 px-3.5 py-2 rounded-lg text-xs font-semibold transition-all ${
              activeSubTab === "ide"
                ? "bg-primary/20 text-primary border border-primary/40 shadow-sm"
                : "text-slate-400 hover:text-white hover:bg-surface-elevated"
            }`}
          >
            <Code2 className="w-4 h-4" />
            <span>Редакторы (Cursor, Windsurf, VS Code)</span>
          </button>

          <button
            onClick={() => setActiveSubTab("ssl")}
            className={`flex items-center gap-2 px-3.5 py-2 rounded-lg text-xs font-semibold transition-all ${
              activeSubTab === "ssl"
                ? "bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 shadow-sm"
                : "text-slate-400 hover:text-white hover:bg-surface-elevated"
            }`}
          >
            <ShieldCheck className="w-4 h-4" />
            <span>SSL & Сертификаты (Node, Git, Python)</span>
          </button>

          <button
            onClick={() => setActiveSubTab("mcp")}
            className={`flex items-center gap-2 px-3.5 py-2 rounded-lg text-xs font-semibold transition-all ${
              activeSubTab === "mcp"
                ? "bg-primary/20 text-primary border border-primary/40 shadow-sm"
                : "text-slate-400 hover:text-white hover:bg-surface-elevated"
            }`}
          >
            <Cpu className="w-4 h-4" />
            <span>MCP Серверы (Context Protocol)</span>
          </button>
        </div>
      </div>

      {/* Notification Toast */}
      {notification && (
        <div
          className={`p-4 rounded-xl border flex items-center justify-between gap-3 text-xs animate-in fade-in ${
            notification.type === "success"
              ? "bg-emerald-500/15 border-emerald-500/30 text-emerald-200"
              : "bg-rose-500/15 border-rose-500/30 text-rose-200"
          }`}
        >
          <div className="flex items-center gap-2">
            {notification.type === "success" ? <Check className="w-4 h-4" /> : <AlertTriangle className="w-4 h-4" />}
            <span className="font-medium whitespace-pre-line">{notification.message}</span>
          </div>
          <button onClick={() => setNotification(null)} className="opacity-60 hover:opacity-100 text-xs font-bold">
            ✕
          </button>
        </div>
      )}

      {/* Subtab 1: IDE Doctor */}
      {activeSubTab === "ide" && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {ideAudits.map((ide) => {
            const isInstalled = ide.is_installed;
            const isHealthy = ide.status === "HEALTHY";
            const canFix = ["cursor", "windsurf", "vscode", "antigravity"].includes(ide.ide_id);

            return (
              <div
                key={ide.ide_id}
                className="rounded-2xl bg-surface border border-surface-border p-5 shadow-lg space-y-3.5 hover:border-primary/40 transition-all flex flex-col justify-between"
              >
                <div className="space-y-3">
                  <div className="flex items-start justify-between">
                    <div className="flex items-center gap-2.5">
                      <div className="p-2 rounded-lg bg-surface-elevated border border-surface-border">
                        <FileCode2 className="w-5 h-5 text-primary" />
                      </div>
                      <div>
                        <h3 className="text-sm font-bold text-white">{ide.ide_name}</h3>
                        <span className="text-[11px] font-mono text-slate-400">
                          {isInstalled ? "Обнаружена в системе" : "Не установлена"}
                        </span>
                      </div>
                    </div>

                    <span
                      className={`text-[10px] font-mono uppercase px-2 py-0.5 rounded-full border ${
                        !isInstalled
                          ? "bg-slate-800 text-slate-500 border-surface-border"
                          : isHealthy
                          ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/30"
                          : "bg-amber-500/10 text-amber-400 border-amber-500/30"
                      }`}
                    >
                      {!isInstalled ? "ОТСУТСТВУЕТ" : isHealthy ? "В НОРМЕ" : "ВНИМАНИЕ"}
                    </span>
                  </div>

                  <div className="text-xs text-slate-300 bg-slate-950/50 p-3 rounded-xl border border-surface-border leading-relaxed">
                    {ide.diagnosis_ru}
                  </div>

                  {ide.config_path && (
                    <div className="text-[11px] font-mono text-slate-400 truncate bg-surface-elevated/40 p-2 rounded-lg border border-surface-border/60">
                      <span className="text-slate-500">Конфиг: </span>
                      {ide.config_path}
                    </div>
                  )}

                  {ide.recommendation_ru && (
                    <div className="text-xs text-amber-300 bg-amber-500/10 p-2.5 rounded-lg border border-amber-500/25">
                      {ide.recommendation_ru}
                    </div>
                  )}
                </div>

                {/* 1-Click Fix Buttons */}
                {canFix && (
                  <div className="pt-3 border-t border-surface-border/60 flex items-center gap-2">
                    <button
                      onClick={() => handleFixIde(ide.ide_id)}
                      disabled={actionLoading === `fix_${ide.ide_id}`}
                      className="flex-1 px-3 py-2 rounded-lg bg-primary/20 hover:bg-primary/30 border border-primary/40 text-primary text-xs font-semibold flex items-center justify-center gap-1.5 transition-all active:scale-95 disabled:opacity-50"
                    >
                      <Zap className="w-3.5 h-3.5" />
                      <span>{actionLoading === `fix_${ide.ide_id}` ? "Настройка..." : "Привязать к мосту (53200)"}</span>
                    </button>

                    <button
                      onClick={() => handleResetIde(ide.ide_id)}
                      disabled={actionLoading === `reset_${ide.ide_id}`}
                      className="px-3 py-2 rounded-lg bg-surface-elevated hover:bg-slate-700/60 border border-border/60 text-slate-400 hover:text-white text-xs font-semibold transition-all active:scale-95 disabled:opacity-50"
                      title="Сбросить параметры прокси к исходным"
                    >
                      <span>{actionLoading === `reset_${ide.ide_id}` ? "..." : "Сбросить"}</span>
                    </button>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {/* Subtab 2: SSL & Dev Certs Doctor */}
      {activeSubTab === "ssl" && (
        <div className="space-y-5">
          <div className="rounded-2xl bg-surface border border-surface-border p-6 space-y-4">
            <div className="flex items-start justify-between gap-4">
              <div>
                <h3 className="text-base font-bold text-white flex items-center gap-2">
                  <ShieldCheck className="w-5 h-5 text-emerald-400" />
                  Устранение ошибок SSL и самоподписанных сертификатов
                </h3>
                <p className="text-xs text-slate-300 mt-1 max-w-2xl leading-relaxed">
                  Решает распространенные ошибки разработчиков при работе через корпоративные сети,
                  VPN или DPI-мосты: <code>UNABLE_TO_GET_ISSUER_CERT_LOCALLY</code>,{" "}
                  <code>self-signed certificate in certificate chain</code> и сбои в <code>git clone</code>.
                </p>
              </div>

              <span
                className={`text-[10px] uppercase font-bold px-2.5 py-1 rounded-full border ${
                  devCerts?.is_permissive_mode_enabled
                    ? "bg-emerald-500/20 text-emerald-300 border-emerald-500/40"
                    : "bg-slate-800 text-slate-400 border-slate-700"
                }`}
              >
                {devCerts?.is_permissive_mode_enabled ? "АДАПТИВНЫЙ РЕЖИМ SSL" : "СТРОГИЙ SSL"}
              </span>
            </div>

            <div className="p-3.5 bg-slate-950/60 rounded-xl border border-surface-border text-xs text-slate-300 leading-relaxed">
              {devCerts?.status_summary}
            </div>

            {/* Audit Status Cards */}
            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-3">
              <div className="p-3 bg-slate-900/60 rounded-xl border border-slate-800">
                <span className="text-[10px] text-slate-400 uppercase tracking-wider block mb-1">
                  Node.js TLS Bypass
                </span>
                <span className="text-xs font-mono font-bold text-white">
                  NODE_TLS_REJECT: {devCerts?.node_tls_reject_unauthorized || "1 (Строгий)"}
                </span>
              </div>

              <div className="p-3 bg-slate-900/60 rounded-xl border border-slate-800">
                <span className="text-[10px] text-slate-400 uppercase tracking-wider block mb-1">
                  Git SSL Backend
                </span>
                <span className="text-xs font-mono font-bold text-emerald-400">
                  {devCerts?.git_ssl_backend || "openssl (по умолч.)"}
                </span>
              </div>

              <div className="p-3 bg-slate-900/60 rounded-xl border border-slate-800">
                <span className="text-[10px] text-slate-400 uppercase tracking-wider block mb-1">
                  npm Strict SSL
                </span>
                <span className="text-xs font-mono font-bold text-white">
                  {devCerts?.npm_strict_ssl || "true"}
                </span>
              </div>

              <div className="p-3 bg-slate-900/60 rounded-xl border border-slate-800">
                <span className="text-[10px] text-slate-400 uppercase tracking-wider block mb-1">
                  Python HTTPS Verify
                </span>
                <span className="text-xs font-mono font-bold text-sky-400">
                  {devCerts?.python_ssl_cert_file ? "Пользовательский" : "Системный"}
                </span>
              </div>
            </div>

            {/* Action Buttons */}
            <div className="pt-2 flex flex-col sm:flex-row items-center gap-3">
              <button
                onClick={handleApplyDevCerts}
                disabled={actionLoading === "dev_certs_apply"}
                className="w-full sm:w-auto px-5 py-3 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white font-semibold text-xs active:scale-95 transition-all flex items-center justify-center gap-2 shadow-lg shadow-emerald-600/20 disabled:opacity-50"
              >
                <Sparkles className="w-4 h-4" />
                <span>
                  {actionLoading === "dev_certs_apply"
                    ? "Применение..."
                    : "🚀 Включить адаптивный SSL (Windows Schannel + Node bypass)"}
                </span>
              </button>

              <button
                onClick={handleRollbackDevCerts}
                disabled={actionLoading === "dev_certs_rollback"}
                className="w-full sm:w-auto px-5 py-3 rounded-xl bg-surface-elevated hover:bg-slate-700/60 border border-border/60 text-slate-300 font-semibold text-xs active:scale-95 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
              >
                <span>{actionLoading === "dev_certs_rollback" ? "Откат..." : "Вернуть строгий SSL"}</span>
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Subtab 3: MCP Doctor */}
      {activeSubTab === "mcp" && (
        <div className="space-y-4">
          {mcpAudits.length === 0 ? (
            <div className="rounded-2xl bg-surface border border-surface-border p-8 text-center text-slate-400 text-xs">
              Конфигурационных файлов MCP не обнаружено.
            </div>
          ) : (
            mcpAudits.map((mcp, idx) => (
              <div
                key={idx}
                className="rounded-2xl bg-surface border border-surface-border p-5 shadow-lg space-y-4"
              >
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-2 pb-3 border-b border-surface-border">
                  <div className="flex items-center gap-2.5">
                    <Server className="w-5 h-5 text-primary" />
                    <div>
                      <h3 className="text-sm font-bold text-white">{mcp.client_name}</h3>
                      <span className="text-[11px] font-mono text-slate-400">{mcp.file_path}</span>
                    </div>
                  </div>

                  <span
                    className={`text-[10px] font-mono uppercase px-2.5 py-0.5 rounded-full border ${
                      mcp.exists
                        ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/30"
                        : "bg-slate-800 text-slate-500 border-surface-border"
                    }`}
                  >
                    {mcp.exists ? `Активен (${mcp.servers_count} серверов)` : "Файл не создан"}
                  </span>
                </div>

                {mcp.servers.length > 0 ? (
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                    {mcp.servers.map((srv, sIdx) => (
                      <div
                        key={sIdx}
                        className="p-3 rounded-xl bg-slate-950/60 border border-surface-border space-y-2"
                      >
                        <div className="flex items-center justify-between">
                          <span className="text-xs font-bold text-white font-mono">{srv.name}</span>
                          <span className="text-[10px] font-mono uppercase px-2 py-0.5 rounded bg-surface-elevated text-slate-400 border border-surface-border">
                            {srv.transport_type}
                          </span>
                        </div>
                        <p className="text-[11px] text-slate-400">{srv.details_ru}</p>
                      </div>
                    ))}
                  </div>
                ) : mcp.exists ? (
                  <p className="text-xs text-slate-500 italic">
                    В файле не объявлено серверов mcpServers.
                  </p>
                ) : null}
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
};
