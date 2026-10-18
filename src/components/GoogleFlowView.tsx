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
  FolderOpen,
  Puzzle,
  ArrowRight,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { FlowUnlockInfo } from "../types";

export const GoogleFlowView: React.FC = () => {
  const [info, setInfo] = useState<FlowUnlockInfo | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [browserLaunching, setBrowserLaunching] = useState<boolean>(false);
  const [embeddedLaunching, setEmbeddedLaunching] = useState<boolean>(false);
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

  const handleLaunchInBrowser = async () => {
    setBrowserLaunching(true);
    setStatusMessage(null);
    try {
      const res = await invoke<string>("open_flow_in_external_browser");
      setStatusMessage({ text: res, isError: false });
    } catch (err: any) {
      setStatusMessage({
        text: `Ошибка запуска в браузере: ${err.toString()}`,
        isError: true,
      });
    } finally {
      setBrowserLaunching(false);
    }
  };

  const handleOpenExtensionFolder = async () => {
    try {
      const res = await invoke<string>("open_extension_folder");
      setStatusMessage({ text: res, isError: false });
    } catch (err: any) {
      setStatusMessage({
        text: `Ошибка открытия папки: ${err.toString()}`,
        isError: true,
      });
    }
  };

  const handleOpenExtensionsManager = async () => {
    try {
      const res = await invoke<string>("open_extensions_manager_in_browser");
      setStatusMessage({ text: res, isError: false });
    } catch (err: any) {
      setStatusMessage({
        text: `Ошибка открытия менеджера расширений: ${err.toString()}`,
        isError: true,
      });
    }
  };

  const handleLaunchEmbedded = async () => {
    setEmbeddedLaunching(true);
    setStatusMessage(null);
    try {
      const res = await invoke<string>("open_google_flow");
      setStatusMessage({ text: res, isError: false });
      await fetchFlowInfo();
    } catch (err: any) {
      setStatusMessage({
        text: `Не удалось запустить окно: ${err.toString()}`,
        isError: true,
      });
    } finally {
      setEmbeddedLaunching(false);
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
      text: "Файл 'flow-unlocker.user.js' скачан. Если установлен Tampermonkey, он откроется автоматически.",
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
                <span className="px-2.5 py-0.5 text-xs font-semibold rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
                  Обход регионов
                </span>
              </div>
              <p className="text-sm text-slate-400">
                Запуск Google Flow (flow.google.com) в вашем основном браузере с готовым обходом региональных блокировок
              </p>
            </div>
          </div>
        </div>

        <button
          onClick={fetchFlowInfo}
          disabled={loading || browserLaunching}
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

      {/* Hero Primary Card: Launch in Main Browser with Extension */}
      <div className="relative overflow-hidden p-8 rounded-2xl border border-purple-500/30 bg-gradient-to-br from-purple-950/40 via-surface to-slate-900/80 shadow-xl shadow-purple-500/5">
        <div className="absolute top-0 right-0 -mt-10 -mr-10 w-72 h-72 bg-purple-500/10 rounded-full blur-3xl pointer-events-none" />
        <div className="relative z-10 space-y-6">
          <div className="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-6">
            <div className="space-y-2 max-w-2xl">
              <div className="flex items-center gap-2">
                <span className="px-2.5 py-0.5 rounded-full text-xs font-bold bg-purple-500/20 text-purple-300 border border-purple-500/30">
                  Рекомендуемый способ
                </span>
                <span className="text-xs text-slate-400">
                  Браузер: <strong className="text-white">{info?.default_browser_name || "Google Chrome"}</strong>
                </span>
              </div>
              <h2 className="text-2xl font-bold text-white tracking-tight">
                Запустить Google Flow сразу в вашем браузере
              </h2>
              <p className="text-sm text-slate-300 leading-relaxed">
                Открывает <span className="font-mono text-purple-300">flow.google.com</span> прямо в вашем браузере с автоматически подключенным расширением разблокировки.
                Вы остаётесь в своём <strong>авторизованном Google-аккаунте</strong> — никаких белых экранов и зависаний!
              </p>
            </div>

            <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3 w-full lg:w-auto shrink-0">
              <button
                onClick={handleLaunchInBrowser}
                disabled={browserLaunching}
                className="px-8 py-4 rounded-xl bg-gradient-to-r from-purple-600 via-pink-600 to-indigo-600 hover:from-purple-500 hover:via-pink-500 hover:to-indigo-500 text-white font-bold text-base shadow-xl shadow-purple-500/25 active:scale-95 transition-all flex items-center justify-center gap-3 disabled:opacity-50"
              >
                <Globe className={`w-5 h-5 text-white ${browserLaunching ? "animate-spin" : ""}`} />
                {browserLaunching ? "Запуск браузера..." : `🚀 Запустить в ${info?.default_browser_name || "Chrome"}`}
              </button>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-4 border-t border-purple-500/20 text-xs">
            <div className="flex items-start gap-3 bg-surface/40 p-3.5 rounded-xl border border-white/5">
              <ShieldCheck className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
              <div>
                <span className="font-semibold text-white block">Ваш родной аккаунт Google</span>
                <span className="text-slate-400">
                  Сохраняются все ваши куки, логин и доступ к проектам Veo
                </span>
              </div>
            </div>

            <div className="flex items-start gap-3 bg-surface/40 p-3.5 rounded-xl border border-white/5">
              <Sparkles className="w-4 h-4 text-purple-400 shrink-0 mt-0.5" />
              <div>
                <span className="font-semibold text-white block">Авто-подключение расширения</span>
                <span className="text-slate-400">
                  Запускается с флагом --load-extension и патчит доступность
                </span>
              </div>
            </div>

            <div className="flex items-start gap-3 bg-surface/40 p-3.5 rounded-xl border border-white/5">
              <Zap className="w-4 h-4 text-sky-400 shrink-0 mt-0.5" />
              <div>
                <span className="font-semibold text-white block">Без зависаний</span>
                <span className="text-slate-400">
                  Работает на полной скорости движка V8 без ограничений WebView
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Permanent Browser Integration Options */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-5">
        {/* Card 1: 1-Step Chrome Extension */}
        <div className="bg-surface/50 border border-border/50 hover:border-purple-500/40 rounded-2xl p-6 space-y-4 transition-all backdrop-blur-xl flex flex-col justify-between">
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="p-2.5 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/20">
                <Puzzle className="w-5 h-5" />
              </div>
              <span className="px-2 py-0.5 rounded-full text-xs font-bold bg-purple-500/20 text-purple-300">
                Постоянно
              </span>
            </div>

            <div>
              <h3 className="font-bold text-white text-base">
                Навсегда добавить в Chrome / Edge
              </h3>
              <p className="text-xs text-slate-400 mt-1">
                Установите наше готовое расширение в браузер за 10 секунд без магазинов и без Tampermonkey:
              </p>
            </div>

            <ol className="text-xs text-slate-300 space-y-1.5 list-decimal list-inside bg-surface-elevated/40 p-3 rounded-xl border border-border/30">
              <li>Нажмите <strong>«Менеджер расширений»</strong> ниже.</li>
              <li>Включите тумблер <strong>«Режим разработчика»</strong> вверху справа.</li>
              <li>Нажмите <strong>«Открыть папку»</strong> и перетащите её в браузер!</li>
            </ol>
          </div>

          <div className="space-y-2 pt-2">
            <button
              onClick={handleOpenExtensionsManager}
              className="w-full py-2.5 px-3 rounded-xl bg-purple-500/15 hover:bg-purple-500/25 border border-purple-500/30 text-purple-300 text-xs font-semibold flex items-center justify-center gap-2 transition-all active:scale-95"
            >
              <ExternalLink className="w-4 h-4" />
              1. Менеджер расширений в браузере
            </button>
            <button
              onClick={handleOpenExtensionFolder}
              className="w-full py-2 px-3 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 text-xs font-semibold flex items-center justify-center gap-2 transition-all active:scale-95"
            >
              <FolderOpen className="w-4 h-4" />
              2. Открыть готовую папку расширения
            </button>
          </div>
        </div>

        {/* Card 2: 1-Click Bookmarklet */}
        <div className="bg-surface/50 border border-border/50 hover:border-sky-500/40 rounded-2xl p-6 space-y-4 transition-all backdrop-blur-xl flex flex-col justify-between">
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="p-2.5 rounded-lg bg-sky-500/10 text-sky-400 border border-sky-500/20">
                <Zap className="w-5 h-5" />
              </div>
              <span className="px-2 py-0.5 rounded-full text-xs font-semibold bg-sky-500/20 text-sky-300">
                1 клик
              </span>
            </div>

            <div>
              <h3 className="font-bold text-white text-base">
                Букмарклет (Закладка в браузере)
              </h3>
              <p className="text-xs text-slate-400 mt-1">
                Работает вообще без установки расширений в любом браузере:
              </p>
            </div>

            <ol className="text-xs text-slate-300 space-y-1.5 list-decimal list-inside bg-surface-elevated/40 p-3 rounded-xl border border-border/30">
              <li>Скопируйте код букмарклета по кнопке ниже.</li>
              <li>Создайте новую закладку в браузере и вставьте код в поле «URL».</li>
              <li>Нажмите на закладку на странице <span className="text-sky-300 font-mono">flow.google.com</span>!</li>
            </ol>
          </div>

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
                  Код Букмарклета скопирован!
                </>
              ) : (
                <>
                  <Copy className="w-4 h-4" />
                  📋 Скопировать код Букмарклета
                </>
              )}
            </button>
          </div>
        </div>

        {/* Card 3: Tampermonkey Script */}
        <div className="bg-surface/50 border border-border/50 hover:border-emerald-500/40 rounded-2xl p-6 space-y-4 transition-all backdrop-blur-xl flex flex-col justify-between">
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="p-2.5 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                <Code2 className="w-5 h-5" />
              </div>
              <span className="px-2 py-0.5 rounded-full text-xs font-semibold bg-emerald-500/20 text-emerald-300">
                Tampermonkey
              </span>
            </div>

            <div>
              <h3 className="font-bold text-white text-base">
                Для расширения Tampermonkey
              </h3>
              <p className="text-xs text-slate-400 mt-1">
                Если у вас уже установлено расширение Tampermonkey или Violentmonkey:
              </p>
            </div>

            <p className="text-xs text-slate-300 bg-surface-elevated/40 p-3 rounded-xl border border-border/30">
              Скачайте или скопируйте скрипт. При открытии файла <span className="font-mono text-emerald-300">flow-unlocker.user.js</span> Tampermonkey автоматически предложит нажать кнопку «Установить».
            </p>
          </div>

          <div className="flex items-center gap-2 pt-2">
            <button
              onClick={handleDownloadUserScript}
              disabled={!info?.script_code}
              className="flex-1 py-2.5 px-3 rounded-xl bg-emerald-500/15 hover:bg-emerald-500/25 border border-emerald-500/30 text-emerald-300 text-xs font-semibold flex items-center justify-center gap-2 transition-all active:scale-95 disabled:opacity-50"
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
                  Скопировано!
                </>
              ) : (
                <>
                  <Copy className="w-4 h-4" />
                  Копировать код
                </>
              )}
            </button>
          </div>
        </div>
      </div>

      {/* Secondary / Optional: Embedded Window & Technical Script Details */}
      <div className="border border-border/40 rounded-xl overflow-hidden bg-surface/30">
        <button
          onClick={() => setShowCode(!showCode)}
          className="w-full flex items-center justify-between p-4 text-sm font-medium text-slate-300 hover:text-white transition-colors"
        >
          <div className="flex items-center gap-2">
            <Code2 className="w-4 h-4 text-purple-400" />
            <span>Дополнительно: Исходный код скрипта патча & Запуск в окне программы</span>
          </div>
          {showCode ? (
            <ChevronUp className="w-4 h-4 text-slate-400" />
          ) : (
            <ChevronDown className="w-4 h-4 text-slate-400" />
          )}
        </button>

        {showCode && (
          <div className="p-4 pt-0 border-t border-border/30 space-y-4">
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 p-3.5 rounded-xl bg-surface-elevated/40 border border-border/30">
              <div className="text-xs text-slate-300">
                <span className="font-semibold text-white block">Изолированное окно приложения (WebView2)</span>
                <span>Требует повторного входа в Google аккаунт. Если в браузере выше всё работает, это окно использовать не обязательно.</span>
              </div>
              <button
                onClick={handleLaunchEmbedded}
                disabled={embeddedLaunching}
                className="px-4 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 text-xs font-semibold shrink-0 active:scale-95 transition-all disabled:opacity-50"
              >
                {embeddedLaunching ? "Открытие..." : "Открыть в WebView2"}
              </button>
            </div>

            <div className="space-y-1.5">
              <span className="text-xs font-semibold text-slate-300">Исходный код инжектора:</span>
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
