import React, { useState, useEffect } from "react";
import {
  Video,
  Play,
  Sparkles,
  ExternalLink,
  Copy,
  Check,
  ShieldCheck,
  Download,
  Code2,
  RefreshCw,
  Zap,
  ChevronDown,
  ChevronUp,
  MonitorPlay,
  CheckCircle2,
  AlertCircle,
  Globe,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { FlowUnlockInfo } from "../types";

export const GoogleFlowView: React.FC = () => {
  const [info, setInfo] = useState<FlowUnlockInfo | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [launching, setLaunching] = useState<boolean>(false);
  const [statusMessage, setStatusMessage] = useState<{
    text: string;
    isError: boolean;
  } | null>(null);
  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const [showCode, setShowCode] = useState<boolean>(false);

  const fetchFlowInfo = async () => {
    setLoading(true);
    try {
      const res = await invoke<FlowUnlockInfo>("get_google_flow_info");
      setInfo(res);
    } catch (err: any) {
      console.error("Failed to get Google Flow info:", err);
      setStatusMessage({
        text: `Ошибка загрузки информации: ${err.toString()}`,
        isError: true,
      });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchFlowInfo();
  }, []);

  const handleLaunchFlow = async () => {
    setLaunching(true);
    setStatusMessage(null);
    try {
      const res = await invoke<string>("open_google_flow");
      setStatusMessage({ text: res, isError: false });
      await fetchFlowInfo();
    } catch (err: any) {
      setStatusMessage({
        text: `Не удалось запустить Google Flow: ${err.toString()}`,
        isError: true,
      });
    } finally {
      setLaunching(false);
    }
  };

  const handleOpenExternalBrowser = async () => {
    try {
      const res = await invoke<string>("open_flow_in_external_browser");
      setStatusMessage({ text: res, isError: false });
    } catch (err: any) {
      setStatusMessage({
        text: `Ошибка запуска браузера: ${err.toString()}`,
        isError: true,
      });
    }
  };

  const handleCopy = (text: string, key: string) => {
    navigator.clipboard.writeText(text);
    setCopiedKey(key);
    setTimeout(() => setCopiedKey(null), 2500);
  };

  const handleDownloadUserScript = () => {
    if (!info?.script_code) return;
    const blob = new Blob([info.script_code], {
      type: "application/javascript;charset=utf-8",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "flow-unlocker.user.js";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    setStatusMessage({
      text: "Файл 'flow-unlocker.user.js' скачан. Установите его в расширение Tampermonkey или Violentmonkey.",
      isError: false,
    });
  };

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 animate-fadeIn">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 bg-surface/50 border border-border/40 p-6 rounded-2xl backdrop-blur-xl">
        <div className="space-y-1">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-xl bg-gradient-to-tr from-purple-500/20 via-pink-500/20 to-sky-500/20 border border-purple-500/30 text-purple-300">
              <Video className="w-6 h-6" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h1 className="text-2xl font-bold tracking-tight text-white">
                  Google Flow Studio
                </h1>
                <span className="px-2.5 py-0.5 text-xs font-semibold rounded-full bg-gradient-to-r from-purple-500/20 to-pink-500/20 text-purple-300 border border-purple-500/30">
                  Veo & ИИ Видео
                </span>
                <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
                  Без Tampermonkey
                </span>
              </div>
              <p className="text-sm text-slate-400">
                Прямой запуск Google Flow (Veo, Imagen) с автоматической подменой региональных RPC-запросов и проверкой доступности
              </p>
            </div>
          </div>
        </div>

        <button
          onClick={fetchFlowInfo}
          disabled={loading || launching}
          className="flex items-center gap-2 px-4 py-2.5 rounded-xl bg-surface-elevated hover:bg-slate-700/50 border border-border/60 text-slate-200 text-sm font-medium transition-all shadow-sm active:scale-95 disabled:opacity-50"
        >
          <RefreshCw className={`w-4 h-4 ${loading ? "animate-spin" : ""}`} />
          Проверить статус
        </button>
      </div>

      {/* Notification banner */}
      {statusMessage && (
        <div
          className={`p-4 rounded-xl border flex items-center justify-between text-sm transition-all ${
            statusMessage.isError
              ? "bg-rose-500/10 border-rose-500/30 text-rose-300"
              : "bg-emerald-500/10 border-emerald-500/30 text-emerald-300"
          }`}
        >
          <div className="flex items-center gap-3">
            {statusMessage.isError ? (
              <AlertCircle className="w-5 h-5 shrink-0" />
            ) : (
              <CheckCircle2 className="w-5 h-5 shrink-0" />
            )}
            <span>{statusMessage.text}</span>
          </div>
          <button
            onClick={() => setStatusMessage(null)}
            className="text-xs opacity-75 hover:opacity-100 underline ml-4"
          >
            Закрыть
          </button>
        </div>
      )}

      {/* Hero 1-Click Launch Card */}
      <div className="relative overflow-hidden p-8 rounded-2xl border border-purple-500/30 bg-gradient-to-br from-purple-950/40 via-surface to-slate-900/80 shadow-xl shadow-purple-500/5">
        <div className="absolute top-0 right-0 -mt-10 -mr-10 w-64 h-64 bg-purple-500/10 rounded-full blur-3xl pointer-events-none" />
        <div className="relative z-10 space-y-6">
          <div className="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-6">
            <div className="space-y-2 max-w-2xl">
              <div className="flex items-center gap-2">
                <span className="px-2.5 py-0.5 rounded-full text-xs font-bold bg-purple-500/20 text-purple-300 border border-purple-500/30">
                  Встроенный модуль обхода
                </span>
                <span className="text-xs text-slate-400">
                  flow.google.com
                </span>
              </div>
              <h2 className="text-2xl font-bold text-white tracking-tight">
                Запустить Google Flow Studio в 1 клик
              </h2>
              <p className="text-sm text-slate-300 leading-relaxed">
                Нажмите кнопку ниже, чтобы открыть Google Flow в нативном защищенном окне приложения.
                Скрипт разблокировки регионов внедряется автоматически при старте страницы.
                Никаких расширений типа Tampermonkey устанавливать не требуется!
              </p>
            </div>

            <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3 w-full lg:w-auto shrink-0">
              <button
                onClick={handleLaunchFlow}
                disabled={launching}
                className="px-6 py-3.5 rounded-xl bg-gradient-to-r from-purple-600 via-pink-600 to-indigo-600 hover:from-purple-500 hover:via-pink-500 hover:to-indigo-500 text-white font-bold text-sm shadow-lg shadow-purple-500/25 active:scale-95 transition-all flex items-center justify-center gap-2.5 disabled:opacity-50"
              >
                <MonitorPlay className={`w-4 h-4 ${launching ? "animate-spin" : ""}`} />
                {launching ? "Открытие окна..." : "🚀 Открыть в приложении"}
              </button>

              <button
                onClick={handleOpenExternalBrowser}
                className="px-5 py-3.5 rounded-xl bg-surface-elevated hover:bg-slate-700/60 border border-purple-500/30 hover:border-purple-500/60 text-purple-200 font-semibold text-sm shadow-sm active:scale-95 transition-all flex items-center justify-center gap-2"
              >
                <Globe className="w-4 h-4 text-sky-400" />
                🌐 Открыть в основном браузере
              </button>
            </div>
          </div>

          <div className="p-3.5 rounded-xl bg-purple-500/10 border border-purple-500/20 text-xs text-purple-200 flex items-start gap-2.5">
            <Sparkles className="w-4 h-4 text-purple-400 shrink-0 mt-0.5" />
            <span>
              <strong>Важно:</strong> Google Flow требует авторизации в Google-аккаунте. Если в изолированном окне отображается белый экран, откройте Flow в вашем <strong>основном браузере</strong> (где вы уже залогинены в Google) и используйте <strong>1-Click Букмарклет (Закладку)</strong> ниже для моментальной разблокировки в Chrome/Edge!
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-4 border-t border-purple-500/20 text-xs">
            <div className="flex items-start gap-3 bg-surface/40 p-3.5 rounded-xl border border-white/5">
              <ShieldCheck className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
              <div>
                <span className="font-semibold text-white block">Подмена RPC-ответов</span>
                <span className="text-slate-400">
                  Автоматически активирует флаги cPZSdc, KV2T2d, rThb8d, cO7JOb, md9xJf
                </span>
              </div>
            </div>

            <div className="flex items-start gap-3 bg-surface/40 p-3.5 rounded-xl border border-white/5">
              <Sparkles className="w-4 h-4 text-purple-400 shrink-0 mt-0.5" />
              <div>
                <span className="font-semibold text-white block">WIZ_global_data.awbSEf</span>
                <span className="text-slate-400">
                  Включает доступность генеративных ИИ-инструментов Veo и Imagen 3
                </span>
              </div>
            </div>

            <div className="flex items-start gap-3 bg-surface/40 p-3.5 rounded-xl border border-white/5">
              <Zap className="w-4 h-4 text-sky-400 shrink-0 mt-0.5" />
              <div>
                <span className="font-semibold text-white block">Изолированная среда</span>
                <span className="text-slate-400">
                  Работает независимо от установленных в вашей системе браузеров
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Alternative Browser Options */}
      <div className="bg-surface/50 border border-border/50 rounded-2xl p-6 space-y-6 backdrop-blur-xl">
        <div className="space-y-1">
          <h3 className="text-lg font-semibold text-white flex items-center gap-2">
            <Globe className="w-5 h-5 text-sky-400" />
            Использование в вашем обычном браузере (Chrome / Edge / Firefox)
          </h3>
          <p className="text-sm text-slate-400">
            Если вы хотите использовать Google Flow в браузере, где уже выполнен вход в Google-аккаунт,
            выберите один из удобных вариантов ниже:
          </p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
          {/* Card 1: 1-Click Bookmarklet */}
          <div className="bg-surface-elevated/40 border border-border/40 hover:border-sky-500/40 rounded-xl p-5 space-y-4 transition-all">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2.5">
                <span className="p-2 rounded-lg bg-sky-500/10 text-sky-400 border border-sky-500/20">
                  <Zap className="w-4 h-4" />
                </span>
                <div>
                  <h4 className="font-semibold text-white text-sm">
                    Способ 1: Букмарклет (Закладка в браузере)
                  </h4>
                  <span className="text-xs text-emerald-400">
                    Не требует установки расширений
                  </span>
                </div>
              </div>
              <span className="px-2 py-0.5 rounded-full text-xs font-semibold bg-sky-500/20 text-sky-300">
                1 клик
              </span>
            </div>

            <p className="text-xs text-slate-300 leading-relaxed">
              Скопируйте букмарклет, создайте новую закладку в панели закладок вашего браузера и вставьте этот код в поле «URL / Адрес».
              Зайдите на <span className="font-mono text-sky-300">flow.google.com</span> и нажмите на закладку один раз!
            </p>

            <div className="pt-2">
              <button
                onClick={() =>
                  info?.bookmarklet &&
                  handleCopy(info.bookmarklet, "bookmarklet")
                }
                disabled={!info?.bookmarklet}
                className="w-full py-2.5 px-4 rounded-xl bg-sky-500/15 hover:bg-sky-500/25 border border-sky-500/30 text-sky-300 text-xs font-semibold flex items-center justify-center gap-2 transition-all active:scale-95 disabled:opacity-50"
              >
                {copiedKey === "bookmarklet" ? (
                  <>
                    <Check className="w-4 h-4 text-emerald-400" />
                    Букмарклет скопирован в буфер!
                  </>
                ) : (
                  <>
                    <Copy className="w-4 h-4" />
                    Скопировать код Букмарклета
                  </>
                )}
              </button>
            </div>
          </div>

          {/* Card 2: Tampermonkey Script */}
          <div className="bg-surface-elevated/40 border border-border/40 hover:border-purple-500/40 rounded-xl p-5 space-y-4 transition-all">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2.5">
                <span className="p-2 rounded-lg bg-purple-500/10 text-purple-400 border border-purple-500/20">
                  <Code2 className="w-4 h-4" />
                </span>
                <div>
                  <h4 className="font-semibold text-white text-sm">
                    Способ 2: Пользовательский скрипт (.user.js)
                  </h4>
                  <span className="text-xs text-purple-300">
                    Для Tampermonkey / Violentmonkey
                  </span>
                </div>
              </div>
              <span className="px-2 py-0.5 rounded-full text-xs font-semibold bg-purple-500/20 text-purple-300">
                Auto
              </span>
            </div>

            <p className="text-xs text-slate-300 leading-relaxed">
              Если в вашем браузере уже установлено расширение Tampermonkey или Violentmonkey,
              скачайте готовый скрипт или скопируйте его исходный код.
            </p>

            <div className="flex items-center gap-2 pt-2">
              <button
                onClick={handleDownloadUserScript}
                disabled={!info?.script_code}
                className="flex-1 py-2.5 px-3 rounded-xl bg-purple-500/15 hover:bg-purple-500/25 border border-purple-500/30 text-purple-300 text-xs font-semibold flex items-center justify-center gap-2 transition-all active:scale-95 disabled:opacity-50"
              >
                <Download className="w-4 h-4" />
                Скачать .user.js
              </button>

              <button
                onClick={() =>
                  info?.script_code && handleCopy(info.script_code, "script")
                }
                disabled={!info?.script_code}
                className="flex-1 py-2.5 px-3 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 text-xs font-semibold flex items-center justify-center gap-2 transition-all active:scale-95 disabled:opacity-50"
              >
                {copiedKey === "script" ? (
                  <>
                    <Check className="w-4 h-4 text-emerald-400" />
                    Код скопирован!
                  </>
                ) : (
                  <>
                    <Copy className="w-4 h-4" />
                    Скопировать код
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Technical Details Accordion */}
      <div className="border border-border/40 rounded-xl overflow-hidden bg-surface/30">
        <button
          onClick={() => setShowCode(!showCode)}
          className="w-full flex items-center justify-between p-4 text-sm font-medium text-slate-300 hover:text-white transition-colors"
        >
          <div className="flex items-center gap-2">
            <Code2 className="w-4 h-4 text-purple-400" />
            <span>Техническое описание внедряемого скрипта патча</span>
          </div>
          {showCode ? (
            <ChevronUp className="w-4 h-4 text-slate-400" />
          ) : (
            <ChevronDown className="w-4 h-4 text-slate-400" />
          )}
        </button>

        {showCode && (
          <div className="p-4 pt-0 border-t border-border/30 space-y-3">
            <p className="text-xs text-slate-400 leading-relaxed">
              Патч выполняется на этапе <code className="text-sky-300 font-mono">document-start</code> до загрузки основного бандла Google Flow.
              Он перехватывает:
            </p>
            <ul className="text-xs text-slate-300 space-y-1 list-disc list-inside">
              <li>
                <span className="font-mono text-purple-300">WIZ_global_data</span>: Устанавливает флаг <code className="font-mono text-emerald-300">awbSEf = true</code> (активация доступа к генеративным моделям).
              </li>
              <li>
                <span className="font-mono text-purple-300">batchexecute & RPCs</span>: В потоке ответов RPC <code className="font-mono text-sky-300">cPZSdc</code> заполняет слоты 30 и 31 значением 1; для <code className="font-mono text-sky-300">KV2T2d, rThb8d, cO7JOb, md9xJf</code> устанавливает единицы.
              </li>
              <li>
                <span className="font-mono text-purple-300">fetch & XMLHttpRequest</span>: Для <code className="font-mono text-sky-300">fetchToolAvailability</code> и <code className="font-mono text-sky-300">checkAppAvailability</code> возвращает <code className="font-mono text-emerald-300">availabilityState: 'AVAILABLE'</code> и <code className="font-mono text-emerald-300">isCountrySupported: true</code>.
              </li>
            </ul>

            <div className="relative mt-2">
              <pre className="p-4 rounded-xl bg-slate-950/80 border border-slate-800 text-xs font-mono text-slate-300 overflow-x-auto max-h-72">
                {info?.script_code || "// Загрузка кода скрипта..."}
              </pre>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
