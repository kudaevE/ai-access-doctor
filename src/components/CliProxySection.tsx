import React, { useState, useEffect } from "react";
import {
  Terminal,
  RefreshCw,
  Zap,
  RotateCcw,
  CheckCircle2,
  AlertTriangle,
  Copy,
  Check,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { CliProxyStatus } from "../types";

export const CliProxySection: React.FC = () => {
  const [status, setStatus] = useState<CliProxyStatus | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [actionLoading, setActionLoading] = useState<boolean>(false);
  const [proxyUrl, setProxyUrl] = useState<string>("http://127.0.0.1:10809");
  const [msg, setMsg] = useState<{ text: string; isError: boolean } | null>(null);

  const fetchStatus = async () => {
    setLoading(true);
    try {
      const res = await invoke<CliProxyStatus>("get_cli_proxy_info");
      setStatus(res);
      if (res.env_https_proxy) {
        setProxyUrl(res.env_https_proxy);
      } else if (res.git_proxy) {
        setProxyUrl(res.git_proxy);
      }
    } catch (e: any) {
      console.error("Failed to load CLI proxy status:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
  }, []);

  const handleApply = async () => {
    setActionLoading(true);
    setMsg(null);
    try {
      const res = await invoke<string>("apply_cli_proxy", { proxyUrl });
      setMsg({ text: res, isError: false });
      await fetchStatus();
    } catch (e: any) {
      setMsg({ text: `Ошибка: ${e.toString()}`, isError: true });
    } finally {
      setActionLoading(false);
    }
  };

  const handleReset = async () => {
    setActionLoading(true);
    setMsg(null);
    try {
      const res = await invoke<string>("reset_cli_proxy");
      setMsg({ text: res, isError: false });
      await fetchStatus();
    } catch (e: any) {
      setMsg({ text: `Ошибка: ${e.toString()}`, isError: true });
    } finally {
      setActionLoading(false);
    }
  };

  return (
    <div className="bg-surface/50 border border-border/50 rounded-2xl p-6 space-y-5 backdrop-blur-xl">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="p-2.5 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/20">
            <Terminal className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="font-bold text-white text-base">
                CLI & Terminal Proxy Sync (Git, npm, pip, PowerShell)
              </h3>
              <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-purple-500/20 text-purple-300">
                1-Click
              </span>
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Устраняет сбои "В браузере всё работает, а в консоли git clone и npm install падают по таймауту"
            </p>
          </div>
        </div>

        <button
          onClick={fetchStatus}
          disabled={loading}
          className="p-2 rounded-xl bg-surface-elevated hover:bg-slate-700/50 border border-border/50 text-slate-300 text-xs transition-all active:scale-95 disabled:opacity-50"
          title="Обновить статус прокси терминалов"
        >
          <RefreshCw className={`w-4 h-4 ${loading ? "animate-spin" : ""}`} />
        </button>
      </div>

      {msg && (
        <div
          className={`p-3.5 rounded-xl border text-xs flex items-center justify-between ${
            msg.isError
              ? "bg-rose-500/10 border-rose-500/30 text-rose-300"
              : "bg-emerald-500/10 border-emerald-500/30 text-emerald-300"
          }`}
        >
          <span>{msg.text}</span>
          <button onClick={() => setMsg(null)} className="underline opacity-75 hover:opacity-100 ml-4">
            Закрыть
          </button>
        </div>
      )}

      {/* Current State Badges */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 text-xs">
        <div className="p-3 rounded-xl bg-surface-elevated/40 border border-border/40">
          <span className="text-slate-400 block mb-1">Git (http.proxy):</span>
          <span className="font-mono font-semibold text-slate-200">
            {status?.git_proxy || "Не задан (прямой эфир)"}
          </span>
        </div>

        <div className="p-3 rounded-xl bg-surface-elevated/40 border border-border/40">
          <span className="text-slate-400 block mb-1">npm (config proxy):</span>
          <span className="font-mono font-semibold text-slate-200">
            {status?.npm_proxy || "Не задан (прямой эфир)"}
          </span>
        </div>

        <div className="p-3 rounded-xl bg-surface-elevated/40 border border-border/40">
          <span className="text-slate-400 block mb-1">Консоль (HTTPS_PROXY):</span>
          <span className="font-mono font-semibold text-slate-200">
            {status?.env_https_proxy || "Не задан (прямой эфир)"}
          </span>
        </div>
      </div>

      {/* Action Controls */}
      <div className="flex flex-col sm:flex-row items-center gap-3 pt-1">
        <div className="w-full sm:flex-1 relative">
          <input
            type="text"
            value={proxyUrl}
            onChange={(e) => setProxyUrl(e.target.value)}
            placeholder="http://127.0.0.1:10809"
            className="w-full px-4 py-2.5 rounded-xl bg-slate-900 border border-border/60 text-white font-mono text-xs focus:outline-none focus:border-purple-500/60"
          />
        </div>

        <button
          onClick={handleApply}
          disabled={actionLoading || !proxyUrl.trim()}
          className="w-full sm:w-auto px-5 py-2.5 rounded-xl bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 text-white font-semibold text-xs shadow-md shadow-purple-500/20 active:scale-95 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
        >
          <Zap className={`w-3.5 h-3.5 ${actionLoading ? "animate-spin" : ""}`} />
          {actionLoading ? "Настройка..." : "Привязать к терминалам"}
        </button>

        <button
          onClick={handleReset}
          disabled={actionLoading}
          className="w-full sm:w-auto px-4 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-300 font-semibold text-xs active:scale-95 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
          title="Сбросить прокси Git, npm и переменных консоли"
        >
          <RotateCcw className="w-3.5 h-3.5 text-slate-400" />
          Сбросить
        </button>
      </div>
    </div>
  );
};
