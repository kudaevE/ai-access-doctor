import React from "react";
import {
  LayoutDashboard,
  Cpu,
  Network,
  ShieldAlert,
  AlertOctagon,
  History,
  Settings,
  Activity,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  Sparkles,
  Layers,
  Code2,
  Globe,
} from "lucide-react";
import { FullDiagnosticReport } from "../types";

export type TabType =
  | "dashboard"
  | "free_hub"
  | "detective"
  | "services"
  | "comparison"
  | "ide_doctor"
  | "network"
  | "vpn"
  | "errors"
  | "history"
  | "settings";

interface SidebarProps {
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
  report: FullDiagnosticReport | null;
  isRunning: boolean;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeTab,
  setActiveTab,
  report,
  isRunning,
}) => {
  const navItems = [
    {
      id: "dashboard" as TabType,
      label: "Дашборд",
      icon: LayoutDashboard,
      badge: isRunning ? "Тест..." : null,
    },
    {
      id: "free_hub" as TabType,
      label: "Свободный ИИ & DoH",
      icon: Globe,
      badge: "Free",
      badgeColor: "bg-emerald-500/20 text-emerald-300 border border-emerald-500/30",
    },
    {
      id: "detective" as TabType,
      label: "Детектив ошибок",
      icon: Sparkles,
      badge: "P0",
      badgeColor: "bg-primary/20 text-primary border border-primary/30",
    },
    {
      id: "services" as TabType,
      label: "ИИ-сервисы",
      icon: Cpu,
      badge: report?.services
        ? `${report.services.filter((s) => s.status_kind === "Pass" || s.status_kind === "AuthBoundary").length}/${report.services.length}`
        : null,
    },
    {
      id: "comparison" as TabType,
      label: "Сравнение каналов",
      icon: Layers,
      badge: "Matrix",
      badgeColor: "bg-sky-500/20 text-sky-300 border border-sky-500/30",
    },
    {
      id: "ide_doctor" as TabType,
      label: "IDE & MCP Doctor",
      icon: Code2,
    },
    {
      id: "network" as TabType,
      label: "Сеть и Прокси",
      icon: Network,
      badge: report?.proxy.has_conflict ? "Конфликт" : null,
      badgeColor: report?.proxy.has_conflict
        ? "bg-rose-500/20 text-rose-400 border border-rose-500/30"
        : undefined,
    },
    {
      id: "vpn" as TabType,
      label: "VPN и Маршруты",
      icon: ShieldAlert,
      badge: report?.vpn.vpn_tunnel_detected ? "VPN активен" : null,
      badgeColor: "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30",
    },
    {
      id: "errors" as TabType,
      label: "Анализ причин",
      icon: AlertOctagon,
      badge:
        report && report.verdict.failed_checks_count > 0
          ? `${report.verdict.failed_checks_count}`
          : null,
      badgeColor: "bg-rose-500/20 text-rose-400 border border-rose-500/30",
    },
    {
      id: "history" as TabType,
      label: "Отчеты и История",
      icon: History,
    },
    {
      id: "settings" as TabType,
      label: "Настройки",
      icon: Settings,
    },
  ];

  const getStatusIcon = () => {
    if (isRunning) {
      return <Activity className="w-4 h-4 text-primary animate-spin" />;
    }
    if (!report) {
      return <div className="w-2.5 h-2.5 rounded-full bg-slate-500 animate-pulse" />;
    }
    switch (report.verdict.overall_state) {
      case "HEALTHY":
        return <CheckCircle2 className="w-4 h-4 text-emerald-400" />;
      case "WARNING":
        return <AlertTriangle className="w-4 h-4 text-amber-400" />;
      case "CRITICAL_ISSUE":
        return <XCircle className="w-4 h-4 text-rose-400" />;
      default:
        return <div className="w-2.5 h-2.5 rounded-full bg-slate-400" />;
    }
  };

  const getStatusText = () => {
    if (isRunning) return "Диагностика...";
    if (!report) return "Ожидание запуска";
    switch (report.verdict.overall_state) {
      case "HEALTHY":
        return "Система в норме";
      case "WARNING":
        return "Внимание";
      case "CRITICAL_ISSUE":
        return "Обнаружен сбой";
      default:
        return "Не определено";
    }
  };

  return (
    <aside className="w-64 bg-sidebar border-r border-surface-border flex flex-col h-screen select-none shrink-0">
      {/* Brand & App Title */}
      <div className="p-4 border-b border-surface-border/60 flex items-center space-x-3">
        <div className="w-9 h-9 rounded-lg bg-gradient-to-br from-primary via-accent-cyan to-accent-emerald flex items-center justify-center shadow-lg shadow-primary/20">
          <Activity className="w-5 h-5 text-slate-950 stroke-[2.5]" />
        </div>
        <div>
          <h1 className="font-bold text-sm tracking-tight text-white flex items-center gap-1.5">
            AI Access Doctor
            <span className="text-[10px] font-semibold uppercase tracking-wider px-1.5 py-0.5 rounded bg-primary/15 text-primary border border-primary/25">
              v2.0
            </span>
          </h1>
          <p className="text-[11px] text-slate-400">Windows AI Diagnostic Suite</p>
        </div>
      </div>

      {/* Navigation links */}
      <nav className="flex-1 p-2.5 space-y-1 overflow-y-auto">
        <div className="text-[10px] font-semibold uppercase tracking-wider text-slate-500 px-3 py-2">
          Навигация платформы
        </div>
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = activeTab === item.id;
          return (
            <button
              key={item.id}
              onClick={() => setActiveTab(item.id)}
              className={`w-full flex items-center justify-between px-3 py-2 rounded-xl text-xs font-medium transition-all ${
                isActive
                  ? "bg-primary/15 text-primary border border-primary/30 shadow-sm"
                  : "text-slate-300 hover:text-white hover:bg-surface-elevated/70"
              }`}
            >
              <div className="flex items-center space-x-2.5">
                <Icon className={`w-4 h-4 ${isActive ? "text-primary" : "text-slate-400"}`} />
                <span>{item.label}</span>
              </div>
              {item.badge && (
                <span
                  className={`text-[9px] font-mono px-1.5 py-0.5 rounded-md ${
                    item.badgeColor ||
                    "bg-surface-elevated text-slate-400 border border-surface-border"
                  }`}
                >
                  {item.badge}
                </span>
              )}
            </button>
          );
        })}
      </nav>

      {/* Bottom Status Card */}
      <div className="p-3 border-t border-surface-border/60 bg-surface/50">
        <div className="p-2.5 rounded-xl border border-surface-border bg-slate-950/60">
          <div className="flex items-center justify-between mb-1.5">
            <span className="text-[10px] uppercase font-bold text-slate-400 tracking-wider">
              Статус системы:
            </span>
            {getStatusIcon()}
          </div>
          <div className="text-xs font-semibold text-slate-200 truncate">
            {getStatusText()}
          </div>
          {report && (
            <div className="mt-1.5 pt-1.5 border-t border-surface-border/50 text-[10px] text-slate-400 flex justify-between">
              <span>Доказательства:</span>
              <span className="font-mono text-primary font-bold">
                {report.verdict.evidence_strength || "CONFIRMED"}
              </span>
            </div>
          )}
        </div>
      </div>
    </aside>
  );
};
