import React from "react";
import {
  CheckCircle2,
  AlertTriangle,
  XCircle,
  HelpCircle,
  Globe,
  Lock,
  ArrowUpRight,
  Info,
} from "lucide-react";
import { FullDiagnosticReport, ServiceProfileResult } from "../types";

interface AiServicesViewProps {
  report: FullDiagnosticReport | null;
}

export const AiServicesView: React.FC<AiServicesViewProps> = ({ report }) => {
  if (!report) {
    return (
      <div className="p-12 text-center border border-dashed border-surface-border rounded-2xl bg-surface/50">
        <HelpCircle className="w-10 h-10 text-slate-500 mx-auto mb-3" />
        <h3 className="text-sm font-bold text-white">Диагностика еще не выполнена</h3>
        <p className="text-xs text-slate-400 mt-1 max-w-sm mx-auto">
          Нажмите кнопку «Начать диагностику» в правом верхнем углу для проверки доступности эндпоинтов ИИ-сервисов.
        </p>
      </div>
    );
  }

  const getStatusBadge = (svc: ServiceProfileResult) => {
    switch (svc.overall_status) {
      case "pass":
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
            <CheckCircle2 className="w-3.5 h-3.5" />
            <span>Доступен ({svc.latency_ms}ms)</span>
          </span>
        );
      case "warning":
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-semibold bg-amber-500/10 text-amber-400 border border-amber-500/30">
            <AlertTriangle className="w-3.5 h-3.5" />
            <span>Ограничения ({svc.status_kind})</span>
          </span>
        );
      case "fail":
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-semibold bg-rose-500/10 text-rose-400 border border-rose-500/30">
            <XCircle className="w-3.5 h-3.5" />
            <span>Недоступен ({svc.status_kind})</span>
          </span>
        );
      default:
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-semibold bg-slate-500/20 text-slate-400 border border-slate-500/30">
            <HelpCircle className="w-3.5 h-3.5" />
            <span>Не определено</span>
          </span>
        );
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-bold text-white tracking-tight">
          Профили ИИ-сервисов и API-эндпоинтов
        </h2>
        <p className="text-xs text-slate-400 mt-0.5">
          Детальная послойная верификация публичных эндпоинтов Google, OpenAI, Anthropic и GitHub.
        </p>
      </div>

      <div className="space-y-4">
        {report.services.map((svc) => (
          <div
            key={svc.id}
            className="p-5 rounded-2xl bg-surface border border-surface-border hover:border-surface-border/80 transition-all shadow-sm"
          >
            {/* Header */}
            <div className="flex flex-col sm:flex-row sm:items-center justify-between pb-4 border-b border-surface-border gap-2">
              <div>
                <div className="flex items-center gap-2">
                  <h3 className="text-sm font-bold text-white">{svc.name}</h3>
                  <span className="text-[10px] uppercase font-mono px-2 py-0.5 rounded bg-surface-elevated text-slate-300 border border-surface-border">
                    {svc.category}
                  </span>
                </div>
                <div className="text-xs text-slate-400 mt-1 font-mono flex items-center gap-1">
                  <Globe className="w-3 h-3 text-primary" />
                  <span>{svc.primary_endpoint}</span>
                </div>
              </div>
              <div>{getStatusBadge(svc)}</div>
            </div>

            {/* Russian Summary Note */}
            <div className="py-3 text-xs text-slate-300 flex items-start gap-2">
              <Info className="w-4 h-4 text-primary shrink-0 mt-0.5" />
              <div>
                <span>{svc.summary_ru}</span>
                {svc.recommendation_ru && (
                  <p className="text-[11px] text-amber-300/90 mt-1">
                    Совет: {svc.recommendation_ru}
                  </p>
                )}
              </div>
            </div>

            {/* Layered Endpoints Inspection Table */}
            <div className="mt-2 overflow-x-auto rounded-xl border border-surface-border bg-slate-950/60">
              <table className="w-full text-left text-xs">
                <thead>
                  <tr className="border-b border-surface-border text-[10px] uppercase tracking-wider text-slate-300 bg-surface-elevated/40 font-mono">
                    <th className="py-2.5 px-3">Проверяемый URL</th>
                    <th className="py-2.5 px-3">DNS</th>
                    <th className="py-2.5 px-3">TCP 443</th>
                    <th className="py-2.5 px-3">TLS Handshake</th>
                    <th className="py-2.5 px-3">HTTP Код</th>
                    <th className="py-2.5 px-3">Задержка</th>
                    <th className="py-2.5 px-3">Результат слоя</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-surface-border/50 font-mono text-[11px]">
                  {svc.endpoints.map((ep, idx) => (
                    <tr key={idx} className="hover:bg-surface-elevated/20 transition-colors">
                      <td className="py-2.5 px-3 font-medium text-slate-200">
                        <div className="flex items-center gap-1 max-w-xs truncate" title={ep.url}>
                          <span>{ep.url}</span>
                          <ArrowUpRight className="w-3 h-3 text-slate-400" />
                        </div>
                      </td>
                      <td className="py-2.5 px-3">
                        {ep.dns_ok ? (
                          <span className="text-emerald-400">OK</span>
                        ) : (
                          <span className="text-rose-400 font-bold">FAIL</span>
                        )}
                      </td>
                      <td className="py-2.5 px-3">
                        {ep.tcp_ok ? (
                          <span className="text-emerald-400">OK</span>
                        ) : (
                          <span className="text-rose-400 font-bold">FAIL</span>
                        )}
                      </td>
                      <td className="py-2.5 px-3">
                        {ep.tls_ok ? (
                          <span className="text-emerald-400 flex items-center gap-1">
                            <Lock className="w-3 h-3" /> OK
                          </span>
                        ) : (
                          <span className="text-rose-400 font-bold">FAIL</span>
                        )}
                      </td>
                      <td className="py-2.5 px-3">
                        {ep.http_status ? (
                          <span
                            className={`px-1.5 py-0.5 rounded text-[10px] font-bold ${
                              ep.http_status >= 200 && ep.http_status < 300
                                ? "bg-emerald-500/20 text-emerald-300"
                                : ep.http_status === 401 || ep.http_status === 403
                                ? "bg-sky-500/20 text-sky-300"
                                : ep.http_status === 429
                                ? "bg-amber-500/20 text-amber-300"
                                : "bg-rose-500/20 text-rose-300"
                            }`}
                          >
                            {ep.http_status}
                          </span>
                        ) : (
                          <span className="text-slate-300">—</span>
                        )}
                      </td>
                      <td className="py-2.5 px-3 text-slate-300">
                        {ep.latency_ms}ms
                      </td>
                      <td className="py-2.5 px-3 text-[11px] font-sans text-slate-300 max-w-xs truncate" title={ep.details}>
                        {ep.details}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
