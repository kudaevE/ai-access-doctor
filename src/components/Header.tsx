import React from "react";
import {
  Play,
  RotateCw,
  Download,
  ShieldCheck,
  Wifi,
  Sparkles,
} from "lucide-react";
import { FullDiagnosticReport } from "../types";

interface HeaderProps {
  report: FullDiagnosticReport | null;
  isRunning: boolean;
  onRunDiagnostics: () => void;
  onExport: () => void;
  isDemoMode: boolean;
  setIsDemoMode: (val: boolean) => void;
}

export const Header: React.FC<HeaderProps> = ({
  report,
  isRunning,
  onRunDiagnostics,
  onExport,
  isDemoMode,
  setIsDemoMode,
}) => {
  return (
    <header className="h-16 bg-surface/80 backdrop-blur-md border-b border-surface-border px-6 flex items-center justify-between z-10">
      {/* Network Snapshot Badges */}
      <div className="flex items-center space-x-3 text-xs">
        <div className="flex items-center space-x-1.5 px-2.5 py-1 rounded-md bg-surface-elevated border border-surface-border text-slate-300">
          <Wifi className="w-3.5 h-3.5 text-primary" />
          <span className="text-slate-400">Шлюз:</span>
          <span className="font-mono text-slate-200">
            {report?.network.primary_gateway || "Определение..."}
          </span>
        </div>

        {report?.proxy.wininet.enabled && (
          <div className="flex items-center space-x-1.5 px-2.5 py-1 rounded-md bg-amber-500/10 border border-amber-500/30 text-amber-300">
            <span className="w-2 h-2 rounded-full bg-amber-400 animate-pulse" />
            <span>Прокси:</span>
            <span className="font-mono">{report.proxy.wininet.server || "Включен"}</span>
          </div>
        )}

        {report?.vpn.vpn_tunnel_detected && (
          <div className="flex items-center space-x-1.5 px-2.5 py-1 rounded-md bg-emerald-500/10 border border-emerald-500/30 text-emerald-300">
            <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
            <span>VPN активен</span>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center space-x-3">
        {/* Demo Mode Toggle */}
        <div className="flex items-center space-x-2 bg-surface-elevated/80 px-2.5 py-1 rounded-lg border border-surface-border text-xs">
          <Sparkles className="w-3.5 h-3.5 text-accent-purple" />
          <span className="text-slate-400">Режим:</span>
          <button
            onClick={() => setIsDemoMode(!isDemoMode)}
            className={`px-2 py-0.5 rounded text-[11px] font-medium transition-colors ${
              isDemoMode
                ? "bg-accent-purple/20 text-accent-purple border border-accent-purple/40"
                : "bg-surface-border/50 text-slate-300 hover:text-white"
            }`}
          >
            {isDemoMode ? "Эмуляция сбоев" : "Реальный аудит"}
          </button>
        </div>

        {/* Export Button */}
        {report && (
          <button
            onClick={onExport}
            className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg border border-surface-border bg-surface-elevated hover:bg-surface-border/80 text-xs font-medium text-slate-300 hover:text-white transition-all shadow-sm"
            title="Экспортировать отчет в Markdown или JSON"
          >
            <Download className="w-3.5 h-3.5" />
            <span>Экспорт</span>
          </button>
        )}

        {/* Primary Action Button: Начать диагностику */}
        <button
          onClick={onRunDiagnostics}
          disabled={isRunning}
          className={`flex items-center space-x-2 px-4 py-1.5 rounded-lg text-xs font-semibold shadow-md transition-all ${
            isRunning
              ? "bg-slate-800 text-slate-400 cursor-not-allowed"
              : "bg-gradient-to-r from-primary to-accent-cyan hover:from-primary-hover hover:to-accent-cyan text-slate-950 shadow-primary/25 hover:shadow-primary/40 active:scale-95"
          }`}
        >
          {isRunning ? (
            <>
              <RotateCw className="w-3.5 h-3.5 animate-spin" />
              <span>Диагностика...</span>
            </>
          ) : (
            <>
              <Play className="w-3.5 h-3.5 fill-current" />
              <span>Начать диагностику</span>
            </>
          )}
        </button>
      </div>
    </header>
  );
};
