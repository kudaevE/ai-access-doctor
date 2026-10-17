import React from "react";
import {
  Play,
  RotateCw,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  HelpCircle,
  ArrowRight,
  ShieldAlert,
  Server,
  Terminal,
  Copy,
  Check,
} from "lucide-react";
import { FullDiagnosticReport, ProgressUpdate } from "../types";

interface DashboardViewProps {
  report: FullDiagnosticReport | null;
  isRunning: boolean;
  progress: ProgressUpdate | null;
  onRunDiagnostics: () => void;
  onSelectTab: (tab: any) => void;
}

export const DashboardView: React.FC<DashboardViewProps> = ({
  report,
  isRunning,
  progress,
  onRunDiagnostics,
  onSelectTab,
}) => {
  const [copiedAction, setCopiedAction] = React.useState<number | null>(null);

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
        icon: <HelpCircle className="w-8 h-8 text-slate-400" />,
        badge: "Ожидание запуска",
        badgeStyle: "bg-slate-500/20 text-slate-300 border-slate-500/30",
      };
    }
    switch (report.verdict.overall_state) {
      case "HEALTHY":
        return {
          bg: "bg-emerald-950/20 border-emerald-500/40",
          border: "border-emerald-500/40 shadow-lg shadow-emerald-950/30",
          icon: <CheckCircle2 className="w-8 h-8 text-emerald-400" />,
          badge: "Все системы исправны",
          badgeStyle: "bg-emerald-500/20 text-emerald-300 border-emerald-500/40",
        };
      case "WARNING":
        return {
          bg: "bg-amber-950/20 border-amber-500/40",
          border: "border-amber-500/40 shadow-lg shadow-amber-950/30",
          icon: <AlertTriangle className="w-8 h-8 text-amber-400" />,
          badge: "Предупреждение",
          badgeStyle: "bg-amber-500/20 text-amber-300 border-amber-500/40",
        };
      case "CRITICAL_ISSUE":
        return {
          bg: "bg-rose-950/20 border-rose-500/40",
          border: "border-rose-500/40 shadow-lg shadow-rose-950/30",
          icon: <XCircle className="w-8 h-8 text-rose-400" />,
          badge: "Обнаружен сбой доступа",
          badgeStyle: "bg-rose-500/20 text-rose-300 border-rose-500/40",
        };
      default:
        return {
          bg: "bg-slate-900 border-surface-border",
          border: "border-surface-border",
          icon: <HelpCircle className="w-8 h-8 text-slate-400" />,
          badge: "Неизвестный результат",
          badgeStyle: "bg-slate-500/20 text-slate-300 border-slate-500/30",
        };
    }
  };

  const vStyle = getVerdictStyle();

  return (
    <div className="space-y-6">
      {/* Top Hero / Action Section */}
      <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-surface to-surface-elevated border border-surface-border p-6 shadow-xl">
        <div className="absolute top-0 right-0 -mt-10 -mr-10 w-64 h-64 bg-primary/10 rounded-full blur-3xl pointer-events-none" />

        <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 relative z-10">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <span className="text-[11px] font-semibold uppercase tracking-wider px-2 py-0.5 rounded bg-primary/10 text-primary border border-primary/20">
                Комплексная диагностика
              </span>
              {report?.timestamp && (
                <span className="text-xs text-slate-400 font-mono">
                  Последний аудит: {new Date(report.timestamp).toLocaleTimeString()}
                </span>
              )}
            </div>
            <h2 className="text-xl font-bold text-white tracking-tight">
              Диагностика сетевого доступа к инструментам ИИ
            </h2>
            <p className="text-xs text-slate-400 max-w-2xl mt-1">
              Глубокая послойная проверка (DNS, TCP, TLS, Прокси, VPN и эндпоинты) для Google Antigravity,
              Gemini, OpenAI, Claude и GitHub.
            </p>
          </div>

          <button
            onClick={onRunDiagnostics}
            disabled={isRunning}
            className={`group relative flex items-center gap-3 px-6 py-3.5 rounded-xl font-semibold text-sm shadow-xl transition-all duration-200 active:scale-95 ${
              isRunning
                ? "bg-slate-800 text-slate-400 cursor-not-allowed border border-surface-border"
                : "bg-gradient-to-r from-primary via-accent-cyan to-accent-emerald text-slate-950 shadow-primary/20 hover:shadow-primary/40 hover:brightness-110"
            }`}
          >
            {isRunning ? (
              <>
                <RotateCw className="w-5 h-5 animate-spin" />
                <div className="text-left">
                  <div className="text-xs font-bold leading-none">Идёт аудит...</div>
                  <div className="text-[10px] opacity-75 mt-0.5 font-mono">
                    {progress?.percent || 0}% завершено
                  </div>
                </div>
              </>
            ) : (
              <>
                <div className="w-7 h-7 rounded-lg bg-slate-950/20 flex items-center justify-center group-hover:scale-110 transition-transform">
                  <Play className="w-4 h-4 fill-current ml-0.5" />
                </div>
                <div className="text-left">
                  <div className="text-xs font-bold leading-none">Начать диагностику</div>
                  <div className="text-[10px] opacity-80 mt-0.5">В один клик</div>
                </div>
              </>
            )}
          </button>
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

      {/* Primary Root Cause Verdict Card */}
      {report && (
        <div className={`p-6 rounded-2xl border transition-all ${vStyle.bg} ${vStyle.border}`}>
          <div className="flex flex-col md:flex-row items-start justify-between gap-4 pb-4 border-b border-white/10">
            <div className="flex items-start gap-4">
              <div className="p-2.5 rounded-xl bg-slate-950/60 border border-white/10">
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
                <h3 className="text-lg font-bold text-white tracking-tight">
                  {report.verdict.title_ru}
                </h3>
              </div>
            </div>

            {/* Confidence Gauge */}
            <div className="flex items-center gap-3 bg-slate-950/60 px-3.5 py-2 rounded-xl border border-white/10">
              <div className="text-right">
                <div className="text-[10px] uppercase font-semibold text-slate-400 tracking-wider">
                  Достоверность
                </div>
                <div className="text-base font-bold font-mono text-primary">
                  {report.verdict.confidence_percent}%
                </div>
              </div>
              <div className="w-10 h-10 rounded-full border-2 border-primary/30 flex items-center justify-center font-mono text-xs text-primary font-bold">
                ✓
              </div>
            </div>
          </div>

          {/* Explanation in Russian */}
          <div className="py-4 text-xs text-slate-300 leading-relaxed">
            {report.verdict.explanation_ru}
          </div>

          {/* Recommended Corrective Actions */}
          {report.verdict.recommended_actions.length > 0 && (
            <div className="mt-2 pt-3 border-t border-white/10">
              <div className="text-xs font-semibold text-slate-200 mb-2.5 flex items-center gap-1.5">
                <Terminal className="w-3.5 h-3.5 text-primary" />
                <span>Рекомендованные шаги решения:</span>
              </div>
              <div className="space-y-2">
                {report.verdict.recommended_actions.map((act, idx) => (
                  <div
                    key={idx}
                    className="flex items-center justify-between p-2.5 rounded-lg bg-slate-950/50 border border-white/10 text-xs text-slate-200 font-mono group hover:border-primary/40 transition-colors"
                  >
                    <span className="break-all">{act}</span>
                    {act.includes("SetEnvironmentVariable") || act.includes("flushdns") || act.includes("git config") ? (
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

      {/* Metrics Row */}
      {report && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="p-4 rounded-xl bg-surface border border-surface-border">
            <div className="text-[11px] text-slate-400 uppercase tracking-wider font-semibold">
              Успешные проверки
            </div>
            <div className="text-2xl font-bold font-mono text-emerald-400 mt-1">
              {report.verdict.passed_checks_count}
            </div>
            <div className="text-[10px] text-slate-300 mt-1">Эндпоинты в норме</div>
          </div>

          <div className="p-4 rounded-xl bg-surface border border-surface-border">
            <div className="text-[11px] text-slate-400 uppercase tracking-wider font-semibold">
              Предупреждения
            </div>
            <div className="text-2xl font-bold font-mono text-amber-400 mt-1">
              {report.verdict.warning_checks_count}
            </div>
            <div className="text-[10px] text-slate-300 mt-1">Задержки / лимиты</div>
          </div>

          <div className="p-4 rounded-xl bg-surface border border-surface-border">
            <div className="text-[11px] text-slate-400 uppercase tracking-wider font-semibold">
              Критические сбои
            </div>
            <div className="text-2xl font-bold font-mono text-rose-400 mt-1">
              {report.verdict.failed_checks_count}
            </div>
            <div className="text-[10px] text-slate-300 mt-1">Недоступные сервисы</div>
          </div>

          <div className="p-4 rounded-xl bg-surface border border-surface-border">
            <div className="text-[11px] text-slate-400 uppercase tracking-wider font-semibold">
              Стек протоколов
            </div>
            <div className="text-sm font-bold font-mono text-primary mt-2 truncate">
              {report.network.dual_stack.preferred_family}
            </div>
            <div className="text-[10px] text-slate-300 mt-1">
              IPv4: {report.network.dual_stack.ipv4_working ? "Да" : "Нет"} | IPv6: {report.network.dual_stack.ipv6_working ? "Да" : "Нет"}
            </div>
          </div>
        </div>
      )}

      {/* Services Grid Preview */}
      {report && (
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-bold text-white flex items-center gap-2">
              <Server className="w-4 h-4 text-primary" />
              <span>Состояние ключевых ИИ-платформ</span>
            </h3>
            <button
              onClick={() => onSelectTab("services")}
              className="text-xs text-primary hover:text-primary-hover font-semibold flex items-center gap-1 transition-colors"
            >
              <span>Подробнее о сервисах</span>
              <ArrowRight className="w-3.5 h-3.5" />
            </button>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            {report.services.map((svc) => {
              const isPass = svc.overall_status === "pass";
              const isWarn = svc.overall_status === "warning";
              return (
                <div
                  key={svc.id}
                  onClick={() => onSelectTab("services")}
                  className="p-3.5 rounded-xl bg-surface border border-surface-border hover:border-primary/40 transition-all cursor-pointer group"
                >
                  <div className="flex items-start justify-between">
                    <div>
                      <h4 className="text-xs font-bold text-white group-hover:text-primary transition-colors">
                        {svc.name}
                      </h4>
                      <p className="text-[11px] text-slate-300 truncate max-w-[180px]">
                        {svc.category}
                      </p>
                    </div>
                    <span
                      className={`text-[10px] font-mono px-2 py-0.5 rounded-full border ${
                        isPass
                          ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/30"
                          : isWarn
                          ? "bg-amber-500/10 text-amber-400 border-amber-500/30"
                          : "bg-rose-500/10 text-rose-400 border-rose-500/30"
                      }`}
                    >
                      {svc.latency_ms > 0 ? `${svc.latency_ms}ms` : svc.overall_status.toUpperCase()}
                    </span>
                  </div>
                  <p className="text-[11px] text-slate-300 mt-2 line-clamp-2">
                    {svc.summary_ru}
                  </p>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};
