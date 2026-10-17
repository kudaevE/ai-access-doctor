import React from "react";
import {
  Network,
  ArrowLeftRight,
  AlertTriangle,
  CheckCircle2,
  XCircle,
  Copy,
  Check,
  Terminal,
  Globe,
  Wifi,
} from "lucide-react";
import { FullDiagnosticReport } from "../types";

interface NetworkProxyViewProps {
  report: FullDiagnosticReport | null;
}

export const NetworkProxyView: React.FC<NetworkProxyViewProps> = ({ report }) => {
  const [copiedText, setCopiedText] = React.useState<string | null>(null);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedText(text);
    setTimeout(() => setCopiedText(null), 2000);
  };

  if (!report) {
    return (
      <div className="p-12 text-center border border-dashed border-surface-border rounded-2xl bg-surface/50">
        <Network className="w-10 h-10 text-slate-500 mx-auto mb-3" />
        <h3 className="text-sm font-bold text-white">Сеть еще не исследована</h3>
        <p className="text-xs text-slate-400 mt-1">
          Запустите диагностику для сбора информации об интерфейсах, прокси и DNS.
        </p>
      </div>
    );
  }

  const { proxy, network } = report;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-bold text-white tracking-tight">
          Сетевые адаптеры и конфигурация прокси
        </h2>
        <p className="text-xs text-slate-400 mt-0.5">
          Аудит системного прокси Windows (WinINet), WinHTTP, переменных окружения и адаптеров.
        </p>
      </div>

      {/* Proxy Conflict Alert Box */}
      {proxy.has_conflict && (
        <div className="p-4 rounded-xl bg-rose-950/20 border border-rose-500/40 text-xs">
          <div className="flex items-start gap-3">
            <div className="p-2 rounded-lg bg-rose-500/10 border border-rose-500/30 text-rose-400 shrink-0">
              <AlertTriangle className="w-5 h-5" />
            </div>
            <div className="space-y-2">
              <h4 className="font-bold text-rose-300 text-sm">
                Обнаружен конфликт конфигурации прокси
              </h4>
              <p className="text-slate-300 leading-relaxed">
                {proxy.conflict_description}
              </p>
              {proxy.recommendation && (
                <div className="pt-2 border-t border-rose-500/20">
                  <span className="font-semibold text-rose-300">Рекомендация: </span>
                  <span className="text-slate-200">{proxy.recommendation}</span>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* 3-Column Proxy Comparison */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Environment Proxy */}
        <div className="p-4 rounded-xl bg-surface border border-surface-border space-y-3">
          <div className="flex items-center justify-between pb-2 border-b border-surface-border">
            <span className="text-xs font-bold text-white flex items-center gap-1.5">
              <Terminal className="w-3.5 h-3.5 text-primary" />
              <span>Переменные окружения</span>
            </span>
            <span
              className={`text-[10px] font-mono px-2 py-0.5 rounded-full border ${
                proxy.env_vars.has_any
                  ? "bg-amber-500/10 text-amber-400 border-amber-500/30"
                  : "bg-slate-500/10 text-slate-400 border-slate-500/30"
              }`}
            >
              {proxy.env_vars.has_any ? "Установлены" : "Не заданы"}
            </span>
          </div>

          <div className="space-y-2 text-xs font-mono">
            <div>
              <span className="text-slate-400 text-[10px] block">HTTP_PROXY:</span>
              <span className="text-slate-200 break-all">
                {proxy.env_vars.http_proxy || "—"}
              </span>
            </div>
            <div>
              <span className="text-slate-400 text-[10px] block">HTTPS_PROXY:</span>
              <span className="text-slate-200 break-all">
                {proxy.env_vars.https_proxy || "—"}
              </span>
            </div>
            <div>
              <span className="text-slate-400 text-[10px] block">ALL_PROXY:</span>
              <span className="text-slate-200 break-all">
                {proxy.env_vars.all_proxy || "—"}
              </span>
            </div>
            <div>
              <span className="text-slate-400 text-[10px] block">NO_PROXY:</span>
              <span className="text-slate-200 break-all">
                {proxy.env_vars.no_proxy || "—"}
              </span>
            </div>
          </div>
        </div>

        {/* Windows User Proxy (WinINet) */}
        <div className="p-4 rounded-xl bg-surface border border-surface-border space-y-3">
          <div className="flex items-center justify-between pb-2 border-b border-surface-border">
            <span className="text-xs font-bold text-white flex items-center gap-1.5">
              <Globe className="w-3.5 h-3.5 text-accent-cyan" />
              <span>Windows User Proxy (WinINet)</span>
            </span>
            <span
              className={`text-[10px] font-mono px-2 py-0.5 rounded-full border ${
                proxy.wininet.enabled
                  ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/30"
                  : "bg-slate-500/10 text-slate-400 border-slate-500/30"
              }`}
            >
              {proxy.wininet.enabled ? "Включен" : "Отключен"}
            </span>
          </div>

          <div className="space-y-2 text-xs font-mono">
            <div>
              <span className="text-slate-400 text-[10px] block">Адрес прокси (ProxyServer):</span>
              <span className="text-slate-200 break-all">
                {proxy.wininet.server || "—"}
              </span>
            </div>
            <div>
              <span className="text-slate-400 text-[10px] block">PAC URL (Скрипт):</span>
              <span className="text-slate-200 break-all">
                {proxy.wininet.pac_url || "—"}
              </span>
            </div>
            <div>
              <span className="text-slate-400 text-[10px] block">Исключения (Override):</span>
              <span className="text-slate-200 break-all truncate block" title={proxy.wininet.override_exceptions}>
                {proxy.wininet.override_exceptions || "<local>"}
              </span>
            </div>
          </div>
        </div>

        {/* WinHTTP Proxy */}
        <div className="p-4 rounded-xl bg-surface border border-surface-border space-y-3">
          <div className="flex items-center justify-between pb-2 border-b border-surface-border">
            <span className="text-xs font-bold text-white flex items-center gap-1.5">
              <ArrowLeftRight className="w-3.5 h-3.5 text-accent-emerald" />
              <span>WinHTTP (Службы ОС)</span>
            </span>
            <span
              className={`text-[10px] font-mono px-2 py-0.5 rounded-full border ${
                proxy.winhttp.is_direct
                  ? "bg-slate-500/10 text-slate-400 border-slate-500/30"
                  : "bg-emerald-500/10 text-emerald-400 border-emerald-500/30"
              }`}
            >
              {proxy.winhttp.is_direct ? "Прямой доступ" : "Прокси"}
            </span>
          </div>

          <div className="space-y-2 text-xs font-mono">
            <div>
              <span className="text-slate-400 text-[10px] block">Прокси-сервер WinHTTP:</span>
              <span className="text-slate-200 break-all">
                {proxy.winhttp.proxy_server || "Прямой доступ (без прокси)"}
              </span>
            </div>
            <div>
              <span className="text-slate-400 text-[10px] block">Список обхода:</span>
              <span className="text-slate-200 break-all">
                {proxy.winhttp.bypass_list || "—"}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Active Proxy Endpoints Verification */}
      {proxy.active_proxies.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-sm font-bold text-white flex items-center gap-2">
            <ArrowLeftRight className="w-4 h-4 text-primary" />
            <span>Верификация доступности прокси-портов</span>
          </h3>

          <div className="overflow-x-auto rounded-xl border border-surface-border bg-slate-950/60">
            <table className="w-full text-left text-xs">
              <thead>
                <tr className="border-b border-surface-border text-[10px] uppercase tracking-wider text-slate-300 bg-surface-elevated/40 font-mono">
                  <th className="py-2.5 px-3">Прокси адрес</th>
                  <th className="py-2.5 px-3">TCP Порт</th>
                  <th className="py-2.5 px-3">HTTPS Трафик</th>
                  <th className="py-2.5 px-3">HTTP Код</th>
                  <th className="py-2.5 px-3">Задержка</th>
                  <th className="py-2.5 px-3">Диагностический вывод</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-surface-border/50 font-mono text-[11px]">
                {proxy.active_proxies.map((p, idx) => (
                  <tr key={idx} className="hover:bg-surface-elevated/20 transition-colors">
                    <td className="py-2.5 px-3 font-medium text-slate-200">{p.endpoint}</td>
                    <td className="py-2.5 px-3">
                      {p.is_listening ? (
                        <span className="text-emerald-400">Слушает</span>
                      ) : (
                        <span className="text-rose-400 font-bold">Закрыт</span>
                      )}
                    </td>
                    <td className="py-2.5 px-3">
                      {p.can_forward_https ? (
                        <span className="text-emerald-400">Работает</span>
                      ) : (
                        <span className="text-rose-400 font-bold">Сбой</span>
                      )}
                    </td>
                    <td className="py-2.5 px-3">
                      {p.http_status ? (
                        <span
                          className={`px-1.5 py-0.5 rounded text-[10px] font-bold ${
                            p.http_status === 200
                              ? "bg-emerald-500/20 text-emerald-300"
                              : "bg-rose-500/20 text-rose-300"
                          }`}
                        >
                          {p.http_status}
                        </span>
                      ) : (
                        <span className="text-slate-300">—</span>
                      )}
                    </td>
                    <td className="py-2.5 px-3 text-slate-300">
                      {p.tcp_rtt_ms ? `${p.tcp_rtt_ms}ms` : "—"}
                    </td>
                    <td className="py-2.5 px-3 text-[11px] font-sans text-slate-300">
                      {p.response_error || "Туннелирование HTTPS успешно верифицировано."}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Network Adapters Table */}
      <div className="space-y-3">
        <h3 className="text-sm font-bold text-white flex items-center gap-2">
          <Wifi className="w-4 h-4 text-primary" />
          <span>Сетевые интерфейсы Windows</span>
        </h3>

        <div className="overflow-x-auto rounded-xl border border-surface-border bg-slate-950/60">
          <table className="w-full text-left text-xs">
            <thead>
              <tr className="border-b border-surface-border text-[10px] uppercase tracking-wider text-slate-300 bg-surface-elevated/40 font-mono">
                <th className="py-2.5 px-3">Имя адаптера</th>
                <th className="py-2.5 px-3">Тип</th>
                <th className="py-2.5 px-3">Статус</th>
                <th className="py-2.5 px-3">IPv4 адрес</th>
                <th className="py-2.5 px-3">Шлюз по умолчанию</th>
                <th className="py-2.5 px-3">DNS-серверы</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-surface-border/50 font-mono text-[11px]">
              {network.interfaces.map((iface, idx) => (
                <tr key={idx} className="hover:bg-surface-elevated/20 transition-colors">
                  <td className="py-2.5 px-3 font-medium text-slate-200">
                    <div>{iface.name}</div>
                    <div className="text-[10px] text-slate-400 font-sans truncate max-w-xs" title={iface.description}>
                      {iface.description}
                    </div>
                  </td>
                  <td className="py-2.5 px-3">
                    <span
                      className={`px-1.5 py-0.5 rounded text-[10px] ${
                        iface.is_virtual
                          ? "bg-accent-purple/20 text-accent-purple border border-accent-purple/30"
                          : "bg-surface-elevated text-slate-300"
                      }`}
                    >
                      {iface.adapter_type}
                    </span>
                  </td>
                  <td className="py-2.5 px-3">
                    {iface.is_up ? (
                      <span className="text-emerald-400">Активен (Up)</span>
                    ) : (
                      <span className="text-slate-300">Отключен</span>
                    )}
                  </td>
                  <td className="py-2.5 px-3 text-slate-200">
                    {iface.ipv4.length > 0 ? iface.ipv4.join(", ") : "—"}
                  </td>
                  <td className="py-2.5 px-3 text-slate-300">
                    {iface.gateways.length > 0 ? iface.gateways.join(", ") : "—"}
                  </td>
                  <td className="py-2.5 px-3 text-slate-300">
                    {iface.dns_servers.length > 0 ? iface.dns_servers.join(", ") : "—"}
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
