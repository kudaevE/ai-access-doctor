import React, { useState, useEffect } from "react";
import {
  Zap,
  RotateCcw,
  ShieldCheck,
  ShieldAlert,
  AlertTriangle,
  RefreshCw,
  Cpu,
  CheckCircle2,
  XCircle,
  HelpCircle,
  ChevronDown,
  ChevronUp,
  Server,
  Layers,
  Sparkles,
  ExternalLink,
  Video,
  MonitorPlay,
  Globe,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { AntigravityDoctorReport } from "../types";
import { HostsDoctorSection } from "./HostsDoctorSection";

export const AntigravityDoctorView: React.FC = () => {
  const [data, setData] = useState<AntigravityDoctorReport | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [actionLoading, setActionLoading] = useState<boolean>(false);
  const [flowLaunching, setFlowLaunching] = useState<boolean>(false);
  const [selectedProfile, setSelectedProfile] = useState<string>("ag_relay");
  const [feedbackMsg, setFeedbackMsg] = useState<{
    text: string;
    isError: boolean;
  } | null>(null);
  const [showAdvanced, setShowAdvanced] = useState<boolean>(false);

  const handleLaunchGoogleFlow = async () => {
    setFlowLaunching(true);
    setFeedbackMsg(null);
    try {
      const res = await invoke<string>("open_flow_in_external_browser");
      setFeedbackMsg({ text: res, isError: false });
    } catch (err: any) {
      setFeedbackMsg({
        text: `Ошибка запуска Google Flow в браузере: ${err.toString()}`,
        isError: true,
      });
    } finally {
      setFlowLaunching(false);
    }
  };

  const fetchStatus = async () => {
    setLoading(true);
    try {
      const res = await invoke<AntigravityDoctorReport>(
        "get_antigravity_status"
      );
      setData(res);
    } catch (err: any) {
      console.error("Failed to fetch Antigravity status:", err);
      setFeedbackMsg({
        text: `Ошибка загрузки статуса: ${err.toString()}`,
        isError: true,
      });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
  }, []);

  const handleApply = async () => {
    setActionLoading(true);
    setFeedbackMsg(null);
    try {
      const res = await invoke<string>("apply_antigravity_unlock", {
        profile: selectedProfile,
      });
      setFeedbackMsg({ text: res, isError: false });
      await fetchStatus();
    } catch (err: any) {
      setFeedbackMsg({
        text: `Не удалось применить обход: ${err.toString()}`,
        isError: true,
      });
    } finally {
      setActionLoading(false);
    }
  };

  const handleRollback = async () => {
    setActionLoading(true);
    setFeedbackMsg(null);
    try {
      const res = await invoke<string>("rollback_antigravity_unlock");
      setFeedbackMsg({ text: res, isError: false });
      await fetchStatus();
    } catch (err: any) {
      setFeedbackMsg({
        text: `Ошибка при откате настроек: ${err.toString()}`,
        isError: true,
      });
    } finally {
      setActionLoading(false);
    }
  };

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 bg-surface/50 border border-border/40 p-6 rounded-2xl backdrop-blur-xl">
        <div className="space-y-1">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-xl bg-gradient-to-tr from-sky-500/20 to-indigo-500/20 border border-sky-500/30 text-sky-400">
              <Zap className="w-6 h-6" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h1 className="text-2xl font-bold tracking-tight text-white">
                  Antigravity & Google AI Doctor
                </h1>
                <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-sky-500/10 text-sky-400 border border-sky-500/30">
                  Обход РФ / СНГ
                </span>
              </div>
              <p className="text-sm text-slate-400">
                Автоматическое устранение блокировок DNS, ошибок region-400 и
                настройка Windows NRPT для Google Antigravity
              </p>
            </div>
          </div>
        </div>

        <button
          onClick={fetchStatus}
          disabled={loading || actionLoading}
          className="flex items-center gap-2 px-4 py-2.5 rounded-xl bg-surface-elevated hover:bg-slate-700/50 border border-border/60 text-slate-200 text-sm font-medium transition-all shadow-sm active:scale-95 disabled:opacity-50"
        >
          <RefreshCw className={`w-4 h-4 ${loading ? "animate-spin" : ""}`} />
          Обновить статус
        </button>
      </div>

      {/* Feedback banner */}
      {feedbackMsg && (
        <div
          className={`p-4 rounded-xl border flex items-center justify-between text-sm transition-all animate-fadeIn ${
            feedbackMsg.isError
              ? "bg-rose-500/10 border-rose-500/30 text-rose-300"
              : "bg-emerald-500/10 border-emerald-500/30 text-emerald-300"
          }`}
        >
          <div className="flex items-center gap-3">
            {feedbackMsg.isError ? (
              <XCircle className="w-5 h-5 shrink-0" />
            ) : (
              <CheckCircle2 className="w-5 h-5 shrink-0" />
            )}
            <span>{feedbackMsg.text}</span>
          </div>
          <button
            onClick={() => setFeedbackMsg(null)}
            className="text-xs opacity-70 hover:opacity-100 underline ml-4"
          >
            Закрыть
          </button>
        </div>
      )}

      {/* Live Status Banner */}
      <div
        className={`p-6 rounded-2xl border transition-all ${
          data?.status_badge === "OPTIMAL"
            ? "bg-gradient-to-r from-emerald-950/40 via-surface to-slate-900/60 border-emerald-500/40 shadow-lg shadow-emerald-500/5"
            : data?.status_badge === "WARNING"
            ? "bg-gradient-to-r from-amber-950/40 via-surface to-slate-900/60 border-amber-500/40"
            : "bg-gradient-to-r from-slate-900/40 via-surface to-slate-900/60 border-border/50"
        }`}
      >
        <div className="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-6">
          <div className="flex items-start gap-4">
            <div
              className={`p-3 rounded-2xl shrink-0 mt-0.5 ${
                data?.status_badge === "OPTIMAL"
                  ? "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30"
                  : data?.status_badge === "WARNING"
                  ? "bg-amber-500/20 text-amber-400 border border-amber-500/30"
                  : "bg-slate-800 text-slate-400 border border-slate-700"
              }`}
            >
              {data?.status_badge === "OPTIMAL" ? (
                <ShieldCheck className="w-7 h-7" />
              ) : data?.status_badge === "WARNING" ? (
                <AlertTriangle className="w-7 h-7" />
              ) : (
                <ShieldAlert className="w-7 h-7" />
              )}
            </div>
            <div className="space-y-1">
              <div className="flex items-center gap-2">
                <span className="text-xs uppercase tracking-wider font-semibold text-slate-400">
                  Текущий статус
                </span>
                <span
                  className={`px-2 py-0.5 rounded-full text-xs font-bold ${
                    data?.status_badge === "OPTIMAL"
                      ? "bg-emerald-500/20 text-emerald-300 border border-emerald-500/40"
                      : data?.status_badge === "WARNING"
                      ? "bg-amber-500/20 text-amber-300 border border-amber-500/40"
                      : "bg-slate-700/40 text-slate-400 border border-slate-600/40"
                  }`}
                >
                  {data?.status_badge === "OPTIMAL"
                    ? "АКТИВЕН И ЗАЩИЩЁН"
                    : data?.status_badge === "WARNING"
                    ? "ТРЕБУЕТСЯ ЗАПУСК"
                    : "ОБХОД ВЫКЛЮЧЕН"}
                </span>
              </div>
              <p className="text-base text-slate-200 font-medium leading-relaxed">
                {data?.overall_status_ru || "Определение статуса соединения..."}
              </p>
              <div className="flex flex-wrap items-center gap-3 pt-1 text-xs text-slate-400">
                <span className="flex items-center gap-1.5">
                  <span
                    className={`w-2 h-2 rounded-full ${
                      data?.nrpt_active ? "bg-emerald-400" : "bg-slate-600"
                    }`}
                  />
                  NRPT DNS: {data?.nrpt_active ? "Активен" : "Выключен"}
                </span>
                <span>•</span>
                <span className="flex items-center gap-1.5">
                  <span
                    className={`w-2 h-2 rounded-full ${
                      data?.ag_dns_running || data?.loopback_relay_ready
                        ? "bg-emerald-400"
                        : "bg-slate-600"
                    }`}
                  />
                  Шлюз (ag_dns):{" "}
                  {data?.ag_dns_running || data?.loopback_relay_ready
                    ? "Работает"
                    : "Остановлен"}
                </span>
                <span>•</span>
                <span className="flex items-center gap-1.5">
                  <span
                    className={`w-2 h-2 rounded-full ${
                      data?.ipv4_preferred ? "bg-emerald-400" : "bg-amber-400"
                    }`}
                  />
                  IPv4 Приоритет:{" "}
                  {data?.ipv4_preferred ? "Оптимизирован" : "Стандартный"}
                </span>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-3 w-full lg:w-auto shrink-0">
            <button
              onClick={handleRollback}
              disabled={actionLoading}
              className="flex-1 lg:flex-none flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-slate-800/80 hover:bg-slate-700 border border-slate-700/80 text-slate-300 text-sm font-medium transition-all active:scale-95 disabled:opacity-50"
              title="Удалить правила NRPT и вернуть настройки Windows по умолчанию"
            >
              <RotateCcw className="w-4 h-4 text-slate-400" />
              Вернуть всё как было
            </button>
          </div>
        </div>
      </div>

      {/* Google Flow Studio Banner */}
      <div className="p-5 rounded-2xl border border-purple-500/30 bg-gradient-to-r from-purple-950/40 via-surface to-slate-900/60 flex flex-col md:flex-row items-start md:items-center justify-between gap-4 shadow-lg shadow-purple-500/5">
        <div className="flex items-center gap-3.5">
          <div className="p-3 rounded-xl bg-purple-500/20 text-purple-300 border border-purple-500/30 shrink-0">
            <Video className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="text-base font-bold text-white">
                Google Flow Studio (Veo & Imagen 3)
              </h3>
              <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-purple-500/20 text-purple-300 border border-purple-500/30">
                Встроенный обход
              </span>
            </div>
            <p className="text-xs text-slate-300 mt-0.5">
              Прямой запуск flow.google.com в вашем основном браузере (Chrome / Edge) со встроенным обходом региональных блокировок.
            </p>
          </div>
        </div>

        <button
          onClick={handleLaunchGoogleFlow}
          disabled={flowLaunching}
          className="w-full md:w-auto px-5 py-2.5 rounded-xl bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 text-white font-semibold text-xs shadow-md shadow-purple-500/20 active:scale-95 transition-all flex items-center justify-center gap-2 shrink-0 disabled:opacity-50"
        >
          <Globe className={`w-4 h-4 ${flowLaunching ? "animate-spin" : ""}`} />
          {flowLaunching ? "Запуск браузера..." : "🚀 Запустить Google Flow в браузере"}
        </button>
      </div>

      {/* 1-Click Action Center */}
      <div className="bg-surface/60 border border-border/50 rounded-2xl p-6 space-y-5 backdrop-blur-xl">
        <div className="space-y-1">
          <h2 className="text-lg font-semibold text-white flex items-center gap-2">
            <Sparkles className="w-5 h-5 text-sky-400" />
            Выбор профиля обхода для Antigravity
          </h2>
          <p className="text-sm text-slate-400">
            Выберите желаемый механизм разблокировки. Каждый режим настраивает
            Windows Name Resolution Policy Table (NRPT) так, чтобы весь
            остальной интернет оставался нетронутым.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* Option 1: AG Relay */}
          <div
            onClick={() => setSelectedProfile("ag_relay")}
            className={`p-5 rounded-xl border cursor-pointer transition-all ${
              selectedProfile === "ag_relay"
                ? "bg-sky-500/10 border-sky-500/60 shadow-md shadow-sky-500/10"
                : "bg-surface-elevated/40 border-border/40 hover:border-border/80"
            }`}
          >
            <div className="flex items-center justify-between mb-3">
              <span className="px-2 py-0.5 rounded-full text-xs font-bold bg-sky-500/20 text-sky-300 border border-sky-500/30">
                Рекомендуется
              </span>
              <div
                className={`w-4 h-4 rounded-full border flex items-center justify-center ${
                  selectedProfile === "ag_relay"
                    ? "border-sky-400 bg-sky-500"
                    : "border-slate-600"
                }`}
              >
                {selectedProfile === "ag_relay" && (
                  <div className="w-1.5 h-1.5 rounded-full bg-slate-950" />
                )}
              </div>
            </div>
            <h3 className="font-semibold text-white text-sm mb-1">
              ⚡ Автоматический шлюз
            </h3>
            <p className="text-xs text-slate-400 leading-relaxed">
              Оптимальный маршрут для Google Antigravity и Gemini. Автоматически обходит региональные ограничения и восстанавливает доступ к ИИ.
            </p>
          </div>

          {/* Option 2: Clean DoH */}
          <div
            onClick={() => setSelectedProfile("clean_doh")}
            className={`p-5 rounded-xl border cursor-pointer transition-all ${
              selectedProfile === "clean_doh"
                ? "bg-sky-500/10 border-sky-500/60 shadow-md shadow-sky-500/10"
                : "bg-surface-elevated/40 border-border/40 hover:border-border/80"
            }`}
          >
            <div className="flex items-center justify-between mb-3">
              <span className="px-2 py-0.5 rounded-full text-xs font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                Без прокси
              </span>
              <div
                className={`w-4 h-4 rounded-full border flex items-center justify-center ${
                  selectedProfile === "clean_doh"
                    ? "border-sky-400 bg-sky-500"
                    : "border-slate-600"
                }`}
              >
                {selectedProfile === "clean_doh" && (
                  <div className="w-1.5 h-1.5 rounded-full bg-slate-950" />
                )}
              </div>
            </div>
            <h3 className="font-semibold text-white text-sm mb-1">
              🛡️ Чистый DoH Anycast (Cloudflare & Google)
            </h3>
            <p className="text-xs text-slate-400 leading-relaxed">
              Направляет запросы к Google CloudCode напрямую на 1.1.1.1 и 8.8.8.8
              через NRPT. Устраняет подмену DNS провайдером (185.149.145.6) без
              сторонних серверов.
            </p>
          </div>

          {/* Option 3: User Proxy */}
          <div
            onClick={() => setSelectedProfile("user_proxy")}
            className={`p-5 rounded-xl border cursor-pointer transition-all ${
              selectedProfile === "user_proxy"
                ? "bg-sky-500/10 border-sky-500/60 shadow-md shadow-sky-500/10"
                : "bg-surface-elevated/40 border-border/40 hover:border-border/80"
            }`}
          >
            <div className="flex items-center justify-between mb-3">
              <span className="px-2 py-0.5 rounded-full text-xs font-semibold bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                127.0.0.1:10809
              </span>
              <div
                className={`w-4 h-4 rounded-full border flex items-center justify-center ${
                  selectedProfile === "user_proxy"
                    ? "border-sky-400 bg-sky-500"
                    : "border-slate-600"
                }`}
              >
                {selectedProfile === "user_proxy" && (
                  <div className="w-1.5 h-1.5 rounded-full bg-slate-950" />
                )}
              </div>
            </div>
            <h3 className="font-semibold text-white text-sm mb-1">
              🔌 Свой Прокси (AG_LS_PROXY)
            </h3>
            <p className="text-xs text-slate-400 leading-relaxed">
              Привязывает переменную окружения AG_LS_PROXY к вашему работающему
              прокси (127.0.0.1:10809). Language Server направляет все gRPC
              вызовы через него.
            </p>
          </div>
        </div>

        <div className="flex flex-col sm:flex-row items-center gap-3 pt-2">
          <button
            onClick={handleApply}
            disabled={actionLoading}
            className="w-full sm:w-auto px-6 py-3.5 rounded-xl bg-gradient-to-r from-sky-500 to-indigo-600 hover:from-sky-400 hover:to-indigo-500 text-white font-semibold text-sm shadow-lg shadow-sky-500/20 active:scale-95 transition-all flex items-center justify-center gap-2.5 disabled:opacity-50"
          >
            <Zap className={`w-4 h-4 ${actionLoading ? "animate-spin" : ""}`} />
            {actionLoading
              ? "Применение настроек..."
              : "Применить выбранный обход в 1 клик"}
          </button>
          <span className="text-xs text-slate-400">
            * Потребуется стандартное подтверждение Windows UAC (от имени
            администратора) для настройки NRPT.
          </span>
        </div>
      </div>

      {/* Diagnostics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {/* Card 1: Antigravity IDE */}
        <div className="bg-surface/50 border border-border/50 rounded-xl p-5 space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              Среда разработки
            </span>
            <span
              className={`px-2 py-0.5 rounded-full text-xs font-semibold ${
                data?.ide_installed
                  ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/30"
                  : "bg-rose-500/10 text-rose-400 border border-rose-500/30"
              }`}
            >
              {data?.ide_installed ? "Найдена" : "Не найдена"}
            </span>
          </div>
          <div>
            <h4 className="font-semibold text-white text-sm">Antigravity IDE</h4>
            <p className="text-xs text-slate-400 mt-1 truncate" title={data?.ide_path || ""}>
              {data?.ide_path || "Не установлена в AppData"}
            </p>
          </div>
          <div className="pt-2 border-t border-border/40 text-xs text-slate-400 flex items-center justify-between">
            <span>Версия: {data?.ide_version || "—"}</span>
            <span className="text-emerald-400">Готова к запуску</span>
          </div>
        </div>

        {/* Card 2: Language Server */}
        <div className="bg-surface/50 border border-border/50 rounded-xl p-5 space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              Языковой сервер (gRPC)
            </span>
            <span
              className={`px-2 py-0.5 rounded-full text-xs font-semibold ${
                data?.language_server_installed
                  ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/30"
                  : "bg-amber-500/10 text-amber-400 border border-amber-500/30"
              }`}
            >
              {data?.language_server_installed ? "Готов" : "Отсутствует"}
            </span>
          </div>
          <div>
            <h4 className="font-semibold text-white text-sm">
              language_server.exe
            </h4>
            <p
              className="text-xs text-slate-400 mt-1 truncate"
              title={data?.language_server_path || ""}
            >
              {data?.language_server_path || "resources\\bin"}
            </p>
          </div>
          <div className="pt-2 border-t border-border/40 text-xs text-slate-400 flex items-center justify-between">
            <span>Размер: {data?.language_server_size_mb ? `${data.language_server_size_mb} МБ` : "—"}</span>
            <span className="text-sky-400">Ядро ИИ</span>
          </div>
        </div>

        {/* Card 3: NRPT Status */}
        <div className="bg-surface/50 border border-border/50 rounded-xl p-5 space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              Windows Таблица NRPT
            </span>
            <span
              className={`px-2 py-0.5 rounded-full text-xs font-semibold ${
                data?.nrpt_active
                  ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/30"
                  : "bg-slate-700 text-slate-400 border border-slate-600"
              }`}
            >
              {data?.nrpt_active ? "Активна" : "Не настроена"}
            </span>
          </div>
          <div>
            <h4 className="font-semibold text-white text-sm">
              Раздельное DNS-разрешение
            </h4>
            <p className="text-xs text-slate-400 mt-1">
              {data?.nrpt_rules && data.nrpt_rules.length > 0
                ? `Активно правил: ${data.nrpt_rules.length}`
                : "Правила для Google PA отсутствуют"}
            </p>
          </div>
          <div className="pt-2 border-t border-border/40 text-xs text-slate-400 flex items-center justify-between">
            <span>Пространство: cloudcode-pa</span>
            <span className={data?.nrpt_active ? "text-emerald-400" : "text-slate-500"}>
              {data?.nrpt_active ? "Защищено" : "Прямой эфир"}
            </span>
          </div>
        </div>

        {/* Card 4: Local Relay */}
        <div className="bg-surface/50 border border-border/50 rounded-xl p-5 space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              Локальный шлюз
            </span>
            <span
              className={`px-2 py-0.5 rounded-full text-xs font-semibold ${
                data?.ag_dns_running || data?.loopback_relay_ready
                  ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/30"
                  : "bg-slate-700 text-slate-400 border border-slate-600"
              }`}
            >
              {data?.ag_dns_running || data?.loopback_relay_ready
                ? "Активен"
                : "Не запущен"}
            </span>
          </div>
          <div>
            <h4 className="font-semibold text-white text-sm">
              Безопасный туннель Google AI
            </h4>
            <p className="text-xs text-slate-400 mt-1">
              Локальная пересылка запросов к Google PA
            </p>
          </div>
          <div className="pt-2 border-t border-border/40 text-xs text-slate-400 flex items-center justify-between">
            <span>Канал: 127.65.71.1</span>
            <span className={data?.loopback_relay_ready ? "text-emerald-400" : "text-slate-500"}>
              {data?.loopback_relay_ready ? "Готов к работе" : "Отключен"}
            </span>
          </div>
        </div>

        {/* Card 5: Cloudcode PA Probe */}
        <div className="bg-surface/50 border border-border/50 rounded-xl p-5 space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              Google CloudCode PA
            </span>
            <span
              className={`px-2 py-0.5 rounded-full text-xs font-semibold ${
                data?.cloudcode_direct_status.includes("Доступен") ||
                data?.cloudcode_direct_status.includes("Туннель")
                  ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/30"
                  : "bg-rose-500/10 text-rose-400 border border-rose-500/30"
              }`}
            >
              {data?.cloudcode_rtt_ms ? `${data.cloudcode_rtt_ms} мс` : "Тест"}
            </span>
          </div>
          <div>
            <h4 className="font-semibold text-white text-sm">
              cloudcode-pa.googleapis.com
            </h4>
            <p className="text-xs text-slate-400 mt-1 line-clamp-2">
              {data?.cloudcode_direct_status || "Проверка эндпоинта..."}
            </p>
          </div>
          <div className="pt-2 border-t border-border/40 text-xs text-slate-400 flex items-center justify-between">
            <span>Шлюз моделей: Google</span>
            <span className="text-sky-400">gRPC PA</span>
          </div>
        </div>

        {/* Card 6: Accounts / DNS poisoning */}
        <div className="bg-surface/50 border border-border/50 rounded-xl p-5 space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              DNS Провайдера
            </span>
            <span
              className={`px-2 py-0.5 rounded-full text-xs font-semibold ${
                data?.accounts_ip === "185.149.145.6"
                  ? "bg-rose-500/10 text-rose-400 border border-rose-500/30"
                  : "bg-emerald-500/10 text-emerald-400 border border-emerald-500/30"
              }`}
            >
              {data?.accounts_ip === "185.149.145.6" ? "Заглушка" : "Чистый DNS"}
            </span>
          </div>
          <div>
            <h4 className="font-semibold text-white text-sm">
              accounts.google.com
            </h4>
            <p className="text-xs text-slate-400 mt-1 line-clamp-2">
              {data?.accounts_status || "Проверка разрешения..."}
            </p>
          </div>
          <div className="pt-2 border-t border-border/40 text-xs text-slate-400 flex items-center justify-between">
            <span>IP: {data?.accounts_ip || "—"}</span>
            <span className={data?.accounts_ip === "185.149.145.6" ? "text-rose-400" : "text-emerald-400"}>
              {data?.accounts_ip === "185.149.145.6" ? "РКН 185.149.145.6" : "Google IP"}
            </span>
          </div>
        </div>
      </div>

      {/* Advanced NRPT Rules Accordion */}
      <div className="border border-border/40 rounded-xl overflow-hidden bg-surface/30">
        <button
          onClick={() => setShowAdvanced(!showAdvanced)}
          className="w-full flex items-center justify-between p-4 text-sm font-medium text-slate-300 hover:text-white transition-colors"
        >
          <div className="flex items-center gap-2">
            <Layers className="w-4 h-4 text-sky-400" />
            <span>
              Активные правила Windows NRPT ({data?.nrpt_rules?.length || 0})
            </span>
          </div>
          {showAdvanced ? (
            <ChevronUp className="w-4 h-4 text-slate-400" />
          ) : (
            <ChevronDown className="w-4 h-4 text-slate-400" />
          )}
        </button>

        {showAdvanced && (
          <div className="p-4 pt-0 border-t border-border/30 space-y-3">
            {data?.nrpt_rules && data.nrpt_rules.length > 0 ? (
              <div className="overflow-x-auto">
                <table className="w-full text-left text-xs">
                  <thead className="text-slate-400 border-b border-border/40">
                    <tr>
                      <th className="pb-2 font-medium">Пространство имён (Namespace)</th>
                      <th className="pb-2 font-medium">DNS-серверы / Шлюзы</th>
                      <th className="pb-2 font-medium">Комментарий / Правило</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-border/20 text-slate-300 font-mono">
                    {data.nrpt_rules.map((rule, idx) => (
                      <tr key={idx} className="hover:bg-surface-elevated/30">
                        <td className="py-2.5 text-sky-300">
                          {rule.namespace.join(", ")}
                        </td>
                        <td className="py-2.5 text-emerald-300">
                          {rule.name_servers.join(", ")}
                        </td>
                        <td className="py-2.5 text-slate-400 font-sans">
                          {rule.comment || rule.name}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <p className="text-xs text-slate-400 py-2">
                В системе Windows сейчас нет активных правил NRPT для доменов
                Antigravity.
              </p>
            )}
          </div>
        )}
      </div>

      {/* Hosts Doctor Section */}
      <HostsDoctorSection />

      {/* Educational Callout */}
      <div className="bg-surface/30 border border-border/40 rounded-xl p-5 space-y-3">
        <h4 className="text-sm font-semibold text-white flex items-center gap-2">
          <HelpCircle className="w-4 h-4 text-sky-400" />
          Принцип работы обхода ограничений:
        </h4>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs text-slate-400 leading-relaxed">
          <div className="space-y-1">
            <span className="font-semibold text-slate-200">
              1. Безболезненный NRPT
            </span>
            <p>
              Вместо глобальной смены DNS всего компьютера, Windows
              перенаправляет исключительно запросы к *.googleapis.com на
              чистые резолверы. Ваш обычный интернет и локальная сеть не
              затрагиваются.
            </p>
          </div>
          <div className="space-y-1">
            <span className="font-semibold text-slate-200">
              2. Приоритет IPv4 над IPv6
            </span>
            <p>
              Windows по умолчанию пытается подключиться к Google Cloud через
              IPv6, маршруты которого у многих провайдеров в РФ зависают на
              20–30 секунд. Мы задаём приоритет IPv4 (префикс ::ffff:0:0/96), что
              мгновенно убирает фризы.
            </p>
          </div>
          <div className="space-y-1">
            <span className="font-semibold text-slate-200">
              3. Обход region-400
            </span>
            <p>
              Локальный шлюз принимает запросы от
              Language Server и безопасно передает их в Google PA, скрывая
              заблокированные региональные маркеры.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
