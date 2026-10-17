import React, { useState } from "react";
import {
  Play,
  RotateCw,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  HelpCircle,
  ArrowRight,
  ShieldCheck,
  Server,
  Terminal,
  Copy,
  Check,
  Zap,
  Activity,
  Wrench,
  Clock,
  Sparkles,
} from "lucide-react";
import { FullDiagnosticReport, HistoryRecordSummary, ProgressUpdate, ServiceAvailability } from "../types";

interface DashboardViewProps {
  report: FullDiagnosticReport | null;
  history: HistoryRecordSummary[];
  isRunning: boolean;
  progress: ProgressUpdate | null;
  onRunDiagnostics: () => void;
  onSelectTab: (tab: any) => void;
  onOpenFixModal: () => void;
}

export const DashboardView: React.FC<DashboardViewProps> = ({
  report,
  history,
  isRunning,
  progress,
  onRunDiagnostics,
  onSelectTab,
  onOpenFixModal,
}) => {
  const [copiedAction, setCopiedAction] = useState<number | null>(null);

  const copyToClipboard = (text: string, index: number) => {
    navigator.clipboard.writeText(text);
    setCopiedAction(index);
    setTimeout(() => setCopiedAction(null), 2000);
  };

  const getVerdictStyle = () => {
    if (!report) {
      return {
        bg: "bg-surface-elevated/40 border-surface-border",
        border: "border-surface-border",
        icon: <HelpCircle className="w-7 h-7 text-slate-400" />,
        badge: "ОЖИДАНИЕ АУДИТА",
        badgeStyle: "bg-slate-500/15 text-slate-300 border-slate-500/30",
      };
    }
    switch (report.verdict.overall_state) {
      case "HEALTHY":
        return {
          bg: "bg-emerald-950/20 border-emerald-500/40",
          border: "border-emerald-500/40 shadow-lg shadow-emerald-950/20",
          icon: <CheckCircle2 className="w-7 h-7 text-emerald-400" />,
          badge: "СИСТЕМЫ ИСПРАВНЫ",
          badgeStyle: "bg-emerald-500/15 text-emerald-300 border-emerald-500/35",
        };
      case "WARNING":
        return {
          bg: "bg-amber-950/20 border-amber-500/40",
          border: "border-amber-500/40 shadow-lg shadow-amber-950/20",
          icon: <AlertTriangle className="w-7 h-7 text-amber-400" />,
          badge: "ПРЕДУПРЕЖДЕНИЕ",
          badgeStyle: "bg-amber-500/15 text-amber-300 border-amber-500/35",
        };
      case "CRITICAL_ISSUE":
        return {
          bg: "bg-rose-950/20 border-rose-500/40",
          border: "border-rose-500/40 shadow-lg shadow-rose-950/20",
          icon: <XCircle className="w-7 h-7 text-rose-400" />,
          badge: "ОБНАРУЖЕН СБОЙ",
          badgeStyle: "bg-rose-500/15 text-rose-300 border-rose-500/35",
        };
      default:
        return {
          bg: "bg-slate-900 border-surface-border",
          border: "border-surface-border",
          icon: <HelpCircle className="w-7 h-7 text-slate-400" />,
          badge: "НЕ ОПРЕДЕЛЕНО",
          badgeStyle: "bg-slate-500/15 text-slate-300 border-slate-500/30",
        };
    }
  };

  const getServiceAvailability = (svc: any): ServiceAvailability => {
    switch (svc.status_kind) {
      case "Pass":
        return "AVAILABLE";
      case "AuthBoundary":
        return "AUTH REQUIRED";
      case "RateLimited":
        return "DEGRADED";
      case "Unknown":
        return "UNKNOWN";
      default:
        return "UNAVAILABLE";
    }
  };

  const getAvailabilityBadge = (av: ServiceAvailability, latency: number) => {
    switch (av) {
      case "AVAILABLE":
        return {
          style: "bg-emerald-500/10 text-emerald-400 border-emerald-500/30",
          label: `${latency}ms • ДОСТУПЕН`,
        };
      case "AUTH REQUIRED":
        return {
          style: "bg-sky-500/10 text-sky-400 border-sky-500/30",
          label: `${latency}ms • АВТОРИЗАЦИЯ`,
        };
      case "DEGRADED":
        return {
          style: "bg-amber-500/10 text-amber-400 border-amber-500/30",
          label: `${latency}ms • ДЕГРАДАЦИЯ`,
        };
      case "UNAVAILABLE":
        return {
          style: "bg-rose-500/10 text-rose-400 border-rose-500/30",
          label: "НЕДОСТУПЕН",
        };
      default:
        return {
          style: "bg-slate-800 text-slate-400 border-surface-border",
          label: "НЕИЗВЕСТНО",
        };
    }
  };

  const getStrengthBadge = (strength: string) => {
    switch (strength) {
      case "CONFIRMED_FACT":
        return {
          label: "ПОДТВЕРЖДЕННЫЙ ФАКТ",
          style: "bg-emerald-500/15 text-emerald-300 border-emerald-500/30",
        };
      case "STRONGLY_SUPPORTED":
        return {
          label: "ВЫСОКАЯ ВЕРОЯТНОСТЬ",
          style: "bg-sky-500/15 text-sky-300 border-sky-500/30",
        };
      case "HYPOTHESIS":
        return {
          label: "ГИПОТЕЗА",
          style: "bg-amber-500/15 text-amber-300 border-amber-500/30",
        };
      default:
        return {
          label: "ТРЕБУЕТСЯ АНАЛИЗ",
          style: "bg-slate-500/15 text-slate-300 border-slate-500/30",
        };
    }
  };

  const vStyle = getVerdictStyle();

  // Compute metrics
  const availableServicesCount = report
    ? report.services.filter((s) => s.status_kind === "Pass" || s.status_kind === "AuthBoundary").length
    : 0;

  const totalServicesCount = report?.services.length || 5;

  const avgLatency = report
    ? Math.round(
        report.services.reduce((acc, curr) => acc + curr.latency_ms, 0) /
          Math.max(1, report.services.length)
      )
    : 0;

  return (
    <div className="space-y-6">
      {/* Top Hero / Action Section */}
      <div className="relative overflow-hidden rounded-2xl bg-surface border border-surface-border p-6 shadow-xl">
        <div className="absolute top-0 right-0 -mt-10 -mr-10 w-72 h-72 bg-primary/10 rounded-full blur-3xl pointer-events-none" />

        <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 relative z-10">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <span className="text-[11px] font-semibold uppercase tracking-wider px-2 py-0.5 rounded bg-primary/15 text-primary border border-primary/25">
                Центр диагностики
              </span>
              <span className="text-xs text-slate-400 font-mono">
                AI Access Doctor v2.0
              </span>
            </div>
            <h2 className="text-xl font-bold text-white tracking-tight">
              Мониторинг подключения и состояния AI-инструментов
            </h2>
            <p className="text-xs text-slate-400 max-w-2xl mt-1">
              Глубокая сквозная проверка сетевых уровней (DNS, TCP, TLS, прокси, VPN и эндпоинты)
              для Google Antigravity, Gemini, OpenAI, Claude и GitHub.
            </p>
          </div>

          <div className="flex items-center gap-2.5">
            <button
              onClick={onOpenFixModal}
              className="flex items-center gap-2 px-4 py-3 rounded-xl font-semibold text-xs bg-surface-elevated hover:bg-surface-border text-slate-200 border border-surface-border hover:border-primary/40 shadow-lg transition-all active:scale-95"
            >
              <Wrench className="w-4 h-4 text-primary" />
              <span>Устранение неполадок</span>
            </button>

            <button
              onClick={onRunDiagnostics}
              disabled={isRunning}
              className={`group relative flex items-center gap-2.5 px-6 py-3 rounded-xl font-semibold text-xs shadow-xl transition-all duration-200 active:scale-95 ${
                isRunning
                  ? "bg-slate-800 text-slate-400 cursor-not-allowed border border-surface-border"
                  : "bg-primary text-slate-950 hover:bg-primary-hover shadow-primary/20"
              }`}
            >
              {isRunning ? (
                <>
                  <RotateCw className="w-4 h-4 animate-spin" />
                  <div className="text-left">
                    <div className="text-xs font-bold leading-none">Идёт аудит...</div>
                    <div className="text-[10px] opacity-75 mt-0.5 font-mono">
                      {progress?.percent || 0}%
                    </div>
                  </div>
                </>
              ) : (
                <>
                  <Play className="w-4 h-4 fill-current" />
                  <span>Начать диагностику</span>
                </>
              )}
            </button>
          </div>
        </div>

        {/* Live Progress Bar when running */}
        {isRunning && (
          <div className="mt-5 pt-4 border-t border-surface-border/60">
            <div className="flex items-center justify-between text-xs mb-1.5 font-mono">
              <span className="text-slate-300 flex items-center gap-2">
                <span className="w-2 h-2 rounded-full bg-primary animate-ping" />
                {progress?.message || "Выполнение диагностических тестов..."}
              </span>
              <span className="text-primary font-bold">{progress?.percent || 0}%</span>
            </div>
            <div className="w-full h-2 bg-slate-950 rounded-full overflow-hidden border border-surface-border">
              <div
                className="h-full bg-gradient-to-r from-primary via-accent-cyan to-accent-emerald transition-all duration-300"
                style={{ width: `${progress?.percent || 5}%` }}
              />
            </div>
          </div>
        )}
      </div>

      {/* Antigravity Fast-Access Banner */}
      <div
        onClick={() => onSelectTab("antigravity_doctor")}
        className="p-4 rounded-xl bg-gradient-to-r from-sky-500/10 via-surface to-indigo-500/10 border border-sky-500/30 hover:border-sky-500/60 cursor-pointer transition-all flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 shadow-sm hover:shadow-sky-500/5 group"
      >
        <div className="flex items-center gap-3">
          <div className="p-2.5 rounded-lg bg-sky-500/20 text-sky-400 border border-sky-500/30 group-hover:scale-105 transition-transform">
            <Zap className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="font-semibold text-white text-sm">
                Antigravity & Google AI Unlocker (РФ / СНГ)
              </span>
              <span className="px-1.5 py-0.2 text-[10px] font-bold rounded bg-sky-500/20 text-sky-300 border border-sky-500/30">
                1-Click
              </span>
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Устранение region-400, раздельное DNS-разрешение через Windows NRPT и оптимизация IPv4
            </p>
          </div>
        </div>
        <div className="flex items-center gap-1.5 text-xs font-semibold text-sky-400 group-hover:text-sky-300 shrink-0">
          <span>Открыть модуль</span>
          <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />
        </div>
      </div>

      {/* 5 Top Metric Cards */}
      <div className="grid grid-cols-2 md:grid-cols-5 gap-3.5">
        {/* Card 1: Состояние подключений */}
        <div className="p-4 rounded-xl bg-surface border border-surface-border space-y-1">
          <div className="text-[10px] text-slate-400 uppercase tracking-wider font-semibold">
            Состояние подключений
          </div>
          <div className="flex items-center gap-1.5 pt-1">
            {report?.verdict.overall_state === "HEALTHY" ? (
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            ) : report?.verdict.overall_state === "WARNING" ? (
              <AlertTriangle className="w-4 h-4 text-amber-400" />
            ) : report?.verdict.overall_state === "CRITICAL_ISSUE" ? (
              <XCircle className="w-4 h-4 text-rose-400" />
            ) : (
              <HelpCircle className="w-4 h-4 text-slate-500" />
            )}
            <span className="text-xs font-bold text-white truncate">
              {report
                ? report.verdict.overall_state === "HEALTHY"
                  ? "В норме"
                  : report.verdict.overall_state === "WARNING"
                  ? "Внимание"
                  : "Сбой связи"
                : "Ожидание"}
            </span>
          </div>
          <p className="text-[10px] text-slate-400 truncate">
            {report?.verdict.primary_issue_code || "Нет данных"}
          </p>
        </div>

        {/* Card 2: Доступные AI-сервисы */}
        <div className="p-4 rounded-xl bg-surface border border-surface-border space-y-1">
          <div className="text-[10px] text-slate-400 uppercase tracking-wider font-semibold">
            Доступные AI-сервисы
          </div>
          <div className="text-xl font-bold font-mono text-emerald-400 pt-0.5">
            {report ? `${availableServicesCount}/${totalServicesCount}` : "— / —"}
          </div>
          <p className="text-[10px] text-slate-400">Готовы к работе</p>
        </div>

        {/* Card 3: Активные предупреждения */}
        <div className="p-4 rounded-xl bg-surface border border-surface-border space-y-1">
          <div className="text-[10px] text-slate-400 uppercase tracking-wider font-semibold">
            Предупреждения
          </div>
          <div className="text-xl font-bold font-mono text-amber-400 pt-0.5">
            {report ? report.verdict.warning_checks_count : 0}
          </div>
          <p className="text-[10px] text-slate-400">Лимиты или задержки</p>
        </div>

        {/* Card 4: Средняя задержка */}
        <div className="p-4 rounded-xl bg-surface border border-surface-border space-y-1">
          <div className="text-[10px] text-slate-400 uppercase tracking-wider font-semibold">
            Средняя задержка
          </div>
          <div className="text-xl font-bold font-mono text-primary pt-0.5">
            {report && avgLatency > 0 ? `${avgLatency}ms` : "—"}
          </div>
          <p className="text-[10px] text-slate-400">
            {avgLatency < 100 && avgLatency > 0
              ? "Отличный отклик"
              : avgLatency >= 100
              ? "Умеренный отклик"
              : "Не измерена"}
          </p>
        </div>

        {/* Card 5: Последняя диагностика */}
        <div className="p-4 rounded-xl bg-surface border border-surface-border space-y-1 col-span-2 md:col-span-1">
          <div className="text-[10px] text-slate-400 uppercase tracking-wider font-semibold">
            Последний аудит
          </div>
          <div className="text-xs font-bold text-slate-200 font-mono pt-1.5 truncate">
            {report?.timestamp ? new Date(report.timestamp).toLocaleTimeString() : "Не запускался"}
          </div>
          <p className="text-[10px] text-slate-400">Локальный снимок</p>
        </div>
      </div>

      {/* Primary Root Cause Verdict Card */}
      {report && (
        <div className={`p-6 rounded-2xl border transition-all ${vStyle.bg} ${vStyle.border} space-y-4`}>
          <div className="flex flex-col md:flex-row items-start justify-between gap-4 pb-4 border-b border-white/10">
            <div className="flex items-start gap-3.5">
              <div className="p-2.5 rounded-xl bg-slate-950/70 border border-white/10">
                {vStyle.icon}
              </div>
              <div>
                <div className="flex items-center gap-2 mb-1">
                  <span className={`text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full border ${vStyle.badgeStyle}`}>
                    {vStyle.badge}
                  </span>
                  <span className="text-xs font-mono text-slate-400">
                    Код: <span className="text-slate-200 font-bold">{report.verdict.primary_issue_code}</span>
                  </span>
                </div>
                <h3 className="text-base font-bold text-white tracking-tight">
                  {report.verdict.title_ru}
                </h3>
              </div>
            </div>

            {/* Categorical Strength Badge */}
            <div className="flex items-center gap-2 bg-slate-950/60 px-3 py-1.5 rounded-xl border border-white/10">
              <span className="text-[10px] uppercase font-semibold text-slate-400 tracking-wider">
                Сила доказательств:
              </span>
              <span
                className={`text-[10px] font-mono font-bold px-2 py-0.5 rounded-full border ${
                  getStrengthBadge(report.verdict.evidence_strength || "CONFIRMED_FACT").style
                }`}
              >
                {getStrengthBadge(report.verdict.evidence_strength || "CONFIRMED_FACT").label}
              </span>
            </div>
          </div>

          {/* Explanation in Russian */}
          <div className="text-xs text-slate-300 leading-relaxed">
            {report.verdict.explanation_ru}
          </div>

          {/* Confirmed Facts Bullet List */}
          {report.verdict.confirmed_facts && report.verdict.confirmed_facts.length > 0 && (
            <div className="p-3.5 rounded-xl bg-slate-950/50 border border-white/10 space-y-1.5">
              <div className="text-[11px] font-bold uppercase tracking-wider text-primary flex items-center gap-1.5">
                <CheckCircle2 className="w-3.5 h-3.5" />
                <span>Подтвержденные объективные факты:</span>
              </div>
              {report.verdict.confirmed_facts.map((fact, idx) => (
                <div key={idx} className="flex items-start gap-2 text-xs text-slate-300 pl-1">
                  <span className="text-primary font-bold">›</span>
                  <span>{fact}</span>
                </div>
              ))}
            </div>
          )}

          {/* Recommended Corrective Actions */}
          {report.verdict.recommended_actions.length > 0 && (
            <div className="pt-2 border-t border-white/10 space-y-2">
              <div className="text-xs font-semibold text-slate-200 flex items-center justify-between">
                <span className="flex items-center gap-1.5">
                  <Terminal className="w-3.5 h-3.5 text-primary" />
                  <span>Рекомендованные шаги решения:</span>
                </span>
                <button
                  onClick={onOpenFixModal}
                  className="text-[11px] text-primary hover:text-primary-hover font-semibold flex items-center gap-1"
                >
                  <span>Устранить в Safe Fix Engine</span>
                  <ArrowRight className="w-3 h-3" />
                </button>
              </div>
              <div className="space-y-2">
                {report.verdict.recommended_actions.map((act, idx) => (
                  <div
                    key={idx}
                    className="flex items-center justify-between p-2.5 rounded-lg bg-slate-950/50 border border-white/10 text-xs text-slate-200 font-mono group hover:border-primary/40 transition-colors"
                  >
                    <span className="break-all">{act}</span>
                    {act.includes("SetEnvironmentVariable") ||
                    act.includes("flushdns") ||
                    act.includes("git config") ? (
                      <button
                        onClick={() => copyToClipboard(act, idx)}
                        className="ml-2 p-1.5 rounded bg-surface-elevated hover:bg-surface-border text-slate-400 hover:text-white transition-colors"
                        title="Скопировать команду"
                      >
                        {copiedAction === idx ? (
                          <Check className="w-3.5 h-3.5 text-emerald-400" />
                        ) : (
                          <Copy className="w-3.5 h-3.5" />
                        )}
                      </button>
                    ) : null}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Services Grid with 5 Product Availability States */}
      {report && (
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-bold text-white uppercase tracking-wider flex items-center gap-2">
              <Server className="w-4 h-4 text-primary" />
              <span>Доступность ключевых ИИ-платформ</span>
            </h3>
            <button
              onClick={() => onSelectTab("services")}
              className="text-xs text-primary hover:text-primary-hover font-semibold flex items-center gap-1 transition-colors"
            >
              <span>Все сервисы и эндпоинты</span>
              <ArrowRight className="w-3.5 h-3.5" />
            </button>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-3.5">
            {report.services.map((svc) => {
              const av = getServiceAvailability(svc);
              const badge = getAvailabilityBadge(av, svc.latency_ms);
              return (
                <div
                  key={svc.id}
                  onClick={() => onSelectTab("services")}
                  className="p-4 rounded-xl bg-surface border border-surface-border hover:border-primary/40 transition-all cursor-pointer group space-y-2.5"
                >
                  <div className="flex items-start justify-between">
                    <div>
                      <h4 className="text-xs font-bold text-white group-hover:text-primary transition-colors">
                        {svc.name}
                      </h4>
                      <p className="text-[11px] text-slate-400 truncate max-w-[170px]">
                        {svc.category}
                      </p>
                    </div>
                    <span
                      className={`text-[10px] font-mono font-semibold px-2 py-0.5 rounded-full border ${badge.style}`}
                    >
                      {badge.label}
                    </span>
                  </div>
                  <p className="text-[11px] text-slate-300 line-clamp-2 leading-relaxed">
                    {svc.summary_ru}
                  </p>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Latency History Sparkline & Trends */}
      {history.length > 1 && (
        <div className="p-5 rounded-2xl bg-surface border border-surface-border space-y-3 shadow-lg">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-bold text-white uppercase tracking-wider flex items-center gap-2">
              <Activity className="w-4 h-4 text-primary" />
              <span>История стабильности диагностических запусков</span>
            </h3>
            <span className="text-[11px] font-mono text-slate-400">
              Записей: {history.length}
            </span>
          </div>

          <div className="grid grid-cols-4 md:grid-cols-8 gap-2 pt-1">
            {history.slice(0, 8).map((rec) => {
              const isHealthy = rec.verdict_state === "HEALTHY";
              const isWarning = rec.verdict_state === "WARNING";
              return (
                <div
                  key={rec.id}
                  className="p-2 rounded-lg bg-surface-elevated/40 border border-surface-border/70 text-center space-y-1"
                >
                  <div
                    className={`w-2.5 h-2.5 mx-auto rounded-full ${
                      isHealthy
                        ? "bg-emerald-400 shadow-sm shadow-emerald-400/50"
                        : isWarning
                        ? "bg-amber-400 shadow-sm shadow-amber-400/50"
                        : "bg-rose-400 shadow-sm shadow-rose-400/50"
                    }`}
                  />
                  <div className="text-[10px] font-mono text-slate-400">
                    {new Date(rec.timestamp).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                  </div>
                  <div className="text-[9px] font-bold uppercase truncate text-slate-300">
                    {rec.verdict_state === "HEALTHY" ? "OK" : rec.verdict_state === "WARNING" ? "WARN" : "FAIL"}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};
