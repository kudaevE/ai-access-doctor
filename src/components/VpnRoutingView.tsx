import React from "react";
import {
  Shield,
  Route,
  CheckCircle2,
  AlertTriangle,
  Info,
  Layers,
} from "lucide-react";
import { FullDiagnosticReport } from "../types";

interface VpnRoutingViewProps {
  report: FullDiagnosticReport | null;
}

export const VpnRoutingView: React.FC<VpnRoutingViewProps> = ({ report }) => {
  if (!report) {
    return (
      <div className="p-12 text-center border border-dashed border-surface-border rounded-2xl bg-surface/50">
        <Shield className="w-10 h-10 text-slate-500 mx-auto mb-3" />
        <h3 className="text-sm font-bold text-white">VPN и маршрутизация не исследованы</h3>
        <p className="text-xs text-slate-400 mt-1">
          Запустите диагностику для поиска виртуальных адаптеров и анализа таблицы маршрутов.
        </p>
      </div>
    );
  }

  const { vpn } = report;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-bold text-white tracking-tight">
          VPN-туннели и таблица маршрутизации Windows
        </h2>
        <p className="text-xs text-slate-400 mt-0.5">
          Обнаружение виртуальных адаптеров (WireGuard, Wintun, Tailscale, WARP) и анализ конфликтов метрик.
        </p>
      </div>

      {/* Routing Conflict Warning */}
      {vpn.has_routing_collision && (
        <div className="p-4 rounded-xl bg-amber-950/20 border border-amber-500/40 text-xs">
          <div className="flex items-start gap-3">
            <div className="p-2 rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-400 shrink-0">
              <AlertTriangle className="w-5 h-5" />
            </div>
            <div>
              <h4 className="font-bold text-amber-300 text-sm">
                Конфликт приоритетов маршрута по умолчанию
              </h4>
              <p className="text-slate-300 mt-1 leading-relaxed">
                Обнаружено несколько шлюзов по умолчанию 0.0.0.0/0 с одинаковым значением метрики. Пакеты к ИИ-сервисам могут сбрасываться или отправляться в неверный интерфейс.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Analysis Summary Box */}
      <div className="p-4 rounded-xl bg-surface border border-surface-border flex items-start gap-3">
        <div className="p-2 rounded-lg bg-primary/10 border border-primary/20 text-primary shrink-0">
          <Info className="w-5 h-5" />
        </div>
        <div>
          <h4 className="text-xs font-bold text-white uppercase tracking-wider">
            Статус туннелирования трафика
          </h4>
          <p className="text-xs text-slate-300 mt-1 leading-relaxed">
            {vpn.analysis_summary}
          </p>
        </div>
      </div>

      {/* Virtual Adapters */}
      <div className="space-y-3">
        <h3 className="text-sm font-bold text-white flex items-center gap-2">
          <Shield className="w-4 h-4 text-primary" />
          <span>Обнаруженные виртуальные адаптеры ({vpn.active_virtual_adapters.length})</span>
        </h3>

        {vpn.active_virtual_adapters.length === 0 ? (
          <div className="p-6 rounded-xl bg-surface border border-surface-border text-center text-xs text-slate-400">
            В системе не обнаружено установленных виртуальных сетевых адаптеров VPN.
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {vpn.active_virtual_adapters.map((adap, idx) => (
              <div
                key={idx}
                className="p-4 rounded-xl bg-surface border border-surface-border space-y-2 hover:border-primary/40 transition-all"
              >
                <div className="flex items-start justify-between">
                  <div>
                    <h4 className="text-xs font-bold text-white">{adap.name}</h4>
                    <span className="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-accent-purple/20 text-accent-purple border border-accent-purple/30">
                      {adap.vpn_type}
                    </span>
                  </div>
                  <span
                    className={`text-[10px] font-mono px-2 py-0.5 rounded-full border ${
                      adap.is_active
                        ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/30"
                        : "bg-slate-500/10 text-slate-400 border-slate-500/30"
                    }`}
                  >
                    {adap.is_active ? "Активен (Up)" : "Отключен"}
                  </span>
                </div>
                <p className="text-[11px] text-slate-400 font-mono truncate" title={adap.description}>
                  {adap.description}
                </p>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Default Routes Table */}
      <div className="space-y-3">
        <h3 className="text-sm font-bold text-white flex items-center gap-2">
          <Route className="w-4 h-4 text-primary" />
          <span>Шлюзы по умолчанию и метрики (IPv4 0.0.0.0/0)</span>
        </h3>

        <div className="overflow-x-auto rounded-xl border border-surface-border bg-slate-950/60">
          <table className="w-full text-left text-xs font-mono">
            <thead>
              <tr className="border-b border-surface-border text-[10px] uppercase tracking-wider text-slate-300 bg-surface-elevated/40">
                <th className="py-2.5 px-3">Назначение</th>
                <th className="py-2.5 px-3">Шлюз (NextHop)</th>
                <th className="py-2.5 px-3">Интерфейс</th>
                <th className="py-2.5 px-3">Метрика</th>
                <th className="py-2.5 px-3">Приоритет</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-surface-border/50 text-[11px]">
              {vpn.default_routes.map((rt, idx) => (
                <tr key={idx} className="hover:bg-surface-elevated/20 transition-colors">
                  <td className="py-2.5 px-3 text-slate-200">{rt.destination}</td>
                  <td className="py-2.5 px-3 text-primary font-bold">{rt.gateway}</td>
                  <td className="py-2.5 px-3 text-slate-300">{rt.interface_name}</td>
                  <td className="py-2.5 px-3 text-slate-300">{rt.metric}</td>
                  <td className="py-2.5 px-3">
                    <span
                      className={`px-1.5 py-0.5 rounded text-[10px] font-sans ${
                        idx === 0
                          ? "bg-emerald-500/20 text-emerald-300 font-semibold"
                          : "bg-surface-elevated text-slate-300"
                      }`}
                    >
                      {idx === 0 ? "Основной" : "Резервный"}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
