import React, { useState, useEffect } from "react";
import {
  Wand2,
  RotateCcw,
  CheckCircle2,
  AlertTriangle,
  Zap,
  Shield,
  Cpu,
  Layers,
  ChevronDown,
  ChevronUp,
  Copy,
  Check,
  RefreshCw,
  Sparkles,
  Server,
  ArrowRight,
} from "lucide-react";
import {
  OneClickStatus,
  AutoTroubleshootReport,
  FreeHubReport,
} from "../types";
import { invoke } from "@tauri-apps/api/core";

export const FreeHubView: React.FC = () => {
  const [status, setStatus] = useState<OneClickStatus | null>(null);
  const [autoReport, setAutoReport] = useState<AutoTroubleshootReport | null>(null);
  const [freeHubData, setFreeHubData] = useState<FreeHubReport | null>(null);

  const [isAutoFixing, setIsAutoFixing] = useState<boolean>(false);
  const [isRollingBack, setIsRollingBack] = useState<boolean>(false);
  const [applyingPreset, setApplyingPreset] = useState<string | null>(null);

  const [notification, setNotification] = useState<{ message: string; type: "success" | "error" } | null>(null);
  const [showAdvanced, setShowAdvanced] = useState<boolean>(false);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const loadStatus = async () => {
    try {
      const res = await invoke<OneClickStatus>("get_one_click_status");
      setStatus(res);
    } catch (e) {
      console.error("Failed to load status:", e);
    }
  };

  const loadCatalog = async () => {
    try {
      const res = await invoke<FreeHubReport>("get_free_hub");
      setFreeHubData(res);
    } catch (e) {
      console.error("Failed to load catalog:", e);
    }
  };

  useEffect(() => {
    loadStatus();
    loadCatalog();
  }, []);

  const notify = (msg: string, type: "success" | "error" = "success") => {
    setNotification({ message: msg, type });
    setTimeout(() => setNotification(null), 4000);
  };

  const handleAutoTroubleshoot = async () => {
    setIsAutoFixing(true);
    setAutoReport(null);
    try {
      const rep = await invoke<AutoTroubleshootReport>("auto_troubleshoot_and_select");
      setAutoReport(rep);
      notify("Автоматический подбор успешно выполнен! Лучший вариант применен.");
      await loadStatus();
    } catch (e: any) {
      notify(`Ошибка: ${e}`, "error");
    } finally {
      setIsAutoFixing(false);
    }
  };

  const handleRollback = async () => {
    setIsRollingBack(true);
    try {
      const res = await invoke<string>("rollback_all_settings");
      notify(res);
      setAutoReport(null);
      await loadStatus();
    } catch (e: any) {
      notify(`Ошибка отката: ${e}`, "error");
    } finally {
      setIsRollingBack(false);
    }
  };

  const handleApplyPreset = async (presetId: string) => {
    setApplyingPreset(presetId);
    try {
      const res = await invoke<string>("apply_one_click_preset", { presetId });
      notify(res);
      await loadStatus();
    } catch (e: any) {
      notify(`Ошибка: ${e}`, "error");
    } finally {
      setApplyingPreset(null);
    }
  };

  const handleCopy = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  return (
    <div className="space-y-6 max-w-6xl mx-auto pb-10">
      {/* Toast Notification */}
      {notification && (
        <div
          className={`fixed bottom-6 right-6 z-50 flex items-center gap-3 rounded-xl border px-5 py-3 shadow-2xl backdrop-blur transition-all animate-in slide-in-from-bottom-5 ${
            notification.type === "success"
              ? "border-emerald-500/40 bg-slate-900/95 text-emerald-300"
              : "border-rose-500/40 bg-slate-900/95 text-rose-300"
          }`}
        >
          {notification.type === "success" ? (
            <CheckCircle2 className="h-5 w-5 text-emerald-400" />
          ) : (
            <AlertTriangle className="h-5 w-5 text-rose-400" />
          )}
          <span className="text-sm font-medium">{notification.message}</span>
        </div>
      )}

      {/* TOP HERO: The Auto-Fix & Rollback Station */}
      <div className="relative overflow-hidden rounded-2xl border border-sky-500/30 bg-gradient-to-b from-sky-950/40 via-slate-900/90 to-slate-900/70 p-6 md:p-8 backdrop-blur shadow-2xl">
        <div className="flex flex-col gap-6 lg:flex-row lg:items-center lg:justify-between">
          <div className="max-w-2xl">
            <div className="flex items-center gap-2.5">
              <span className="flex h-8 w-8 items-center justify-center rounded-xl bg-sky-500/20 text-sky-400 shadow-inner">
                <Wand2 className="h-5 w-5" />
              </span>
              <h2 className="text-2xl font-bold tracking-tight text-white">
                Мастер свободного ИИ и DNS
              </h2>
              <span className="rounded-full bg-emerald-500/20 px-3 py-0.5 text-xs font-semibold text-emerald-400 border border-emerald-500/30">
                1-Клик
              </span>
            </div>
            <p className="mt-2 text-sm text-slate-300 leading-relaxed">
              Не нужно ничего настраивать вручную: программа сама протестирует бесплатные нейросети и надежные DNS, выберет действительно работающий для вашего провайдера вариант и сразу подключит его.
            </p>

            {/* Current System State Banner */}
            <div className="mt-4 flex flex-wrap items-center gap-4 text-xs">
              <div className="flex items-center gap-2 rounded-lg bg-slate-950/70 px-3 py-1.5 border border-slate-800 text-slate-300">
                <Shield className="h-3.5 w-3.5 text-sky-400" />
                <span>Текущий DNS:</span>
                <span className="font-mono font-bold text-white">
                  {status?.current_dns.join(", ") || "Авто (DHCP)"}
                </span>
                {status?.is_cloudflare_dns && (
                  <span className="text-[10px] text-emerald-400 font-semibold bg-emerald-500/20 px-1.5 rounded">
                    Cloudflare
                  </span>
                )}
              </div>

              <div className="flex items-center gap-2 rounded-lg bg-slate-950/70 px-3 py-1.5 border border-slate-800 text-slate-300">
                <Zap className="h-3.5 w-3.5 text-amber-400" />
                <span>Активный ИИ:</span>
                <span className="font-mono font-bold text-emerald-400">
                  {status?.active_ai_preset?.toUpperCase() || "Стандартный"}
                </span>
              </div>
            </div>
          </div>

          {/* Action Buttons: Auto-Troubleshoot & Rollback */}
          <div className="flex flex-col sm:flex-row lg:flex-col gap-3 min-w-[280px]">
            {/* Primary Magic Button */}
            <button
              onClick={handleAutoTroubleshoot}
              disabled={isAutoFixing}
              className="group relative flex items-center justify-center gap-2.5 rounded-xl bg-gradient-to-r from-sky-500 to-indigo-600 px-6 py-3.5 text-sm font-bold text-white shadow-lg shadow-sky-500/25 transition-all hover:from-sky-400 hover:to-indigo-500 active:scale-95 disabled:opacity-50"
            >
              <Wand2 className={`h-5 w-5 ${isAutoFixing ? "animate-spin" : "transition-transform group-hover:rotate-12"}`} />
              <span>{isAutoFixing ? "Подбираем лучший вариант..." : "Подобрать и включить рабочий ИИ"}</span>
            </button>

            {/* Rollback Safety Button */}
            <button
              onClick={handleRollback}
              disabled={isRollingBack}
              className="flex items-center justify-center gap-2 rounded-xl border border-rose-500/30 bg-rose-500/10 px-5 py-2.5 text-xs font-semibold text-rose-300 transition-all hover:bg-rose-500/20 active:scale-95 disabled:opacity-50"
              title="Мгновенно восстанавливает все системные настройки до состояния при запуске программы"
            >
              <RotateCcw className={`h-4 w-4 ${isRollingBack ? "animate-spin" : ""}`} />
              <span>{isRollingBack ? "Восстановление..." : "Вернуть всё как было до запуска"}</span>
            </button>
          </div>
        </div>

        {/* Auto Troubleshoot Live Result */}
        {autoReport && (
          <div className="mt-6 rounded-xl border border-emerald-500/40 bg-emerald-950/20 p-5 backdrop-blur animate-in fade-in-50">
            <div className="flex items-start gap-3">
              <CheckCircle2 className="h-5 w-5 text-emerald-400 shrink-0 mt-0.5" />
              <div className="space-y-2">
                <h4 className="font-bold text-emerald-300">{autoReport.title_ru}</h4>
                <p className="text-xs text-slate-300 leading-relaxed">{autoReport.explanation_ru}</p>

                <div className="mt-3 grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2 border-t border-emerald-500/20 text-xs">
                  <div className="rounded-lg bg-slate-900/80 p-2.5 border border-slate-800">
                    <span className="text-slate-400">Подключен свободный ИИ:</span>
                    <div className="mt-0.5 font-bold text-white flex items-center gap-1.5">
                      <Zap className="h-3.5 w-3.5 text-amber-400" />
                      <span>{autoReport.selected_ai_name}</span>
                    </div>
                  </div>

                  <div className="rounded-lg bg-slate-900/80 p-2.5 border border-slate-800">
                    <span className="text-slate-400">Защищенный DNS:</span>
                    <div className="mt-0.5 font-bold text-white flex items-center gap-1.5">
                      <Shield className="h-3.5 w-3.5 text-sky-400" />
                      <span>{autoReport.selected_dns_name}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* 1-CLICK PRESET CARDS: Simple, Visual, Instant */}
      <div>
        <h3 className="text-base font-bold text-white mb-3 flex items-center gap-2">
          <span>Быстрое переключение в 1 клик</span>
          <span className="text-xs font-normal text-slate-400">(нажмите кнопку, чтобы сразу применить)</span>
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* Card 1: Cloudflare DNS */}
          <div className="flex flex-col justify-between rounded-xl border border-slate-800 bg-slate-900/70 p-5 backdrop-blur transition-all hover:border-slate-700">
            <div>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 font-bold text-white">
                  <Shield className="h-4 w-4 text-sky-400" />
                  <span>DNS Cloudflare (1.1.1.1)</span>
                </div>
                {status?.is_cloudflare_dns ? (
                  <span className="rounded-full bg-emerald-500/20 px-2 py-0.5 text-[11px] font-semibold text-emerald-400 border border-emerald-500/30">
                    АКТИВЕН
                  </span>
                ) : (
                  <span className="rounded bg-slate-800 px-2 py-0.5 text-[10px] text-slate-400">
                    Рекомендуется
                  </span>
                )}
              </div>
              <p className="mt-2 text-xs text-slate-400 leading-relaxed">
                Защищает от подмены адресов и блокировок ИИ-сервисов интернет-провайдером.
              </p>
            </div>

            <button
              onClick={() => handleApplyPreset("cloudflare_dns")}
              disabled={applyingPreset !== null || status?.is_cloudflare_dns}
              className={`mt-4 flex w-full items-center justify-center gap-2 rounded-lg py-2.5 text-xs font-bold transition-all ${
                status?.is_cloudflare_dns
                  ? "bg-slate-800 text-slate-400 cursor-default"
                  : "bg-sky-500 text-white hover:bg-sky-400 active:scale-95 shadow-md shadow-sky-500/20"
              }`}
            >
              {status?.is_cloudflare_dns ? (
                <>
                  <Check className="h-4 w-4 text-emerald-400" />
                  <span>Уже включен</span>
                </>
              ) : (
                <>
                  <Shield className="h-4 w-4" />
                  <span>{applyingPreset === "cloudflare_dns" ? "Применение..." : "Включить 1.1.1.1"}</span>
                </>
              )}
            </button>
          </div>

          {/* Card 2: Groq Cloud */}
          <div className="flex flex-col justify-between rounded-xl border border-slate-800 bg-slate-900/70 p-5 backdrop-blur transition-all hover:border-slate-700">
            <div>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 font-bold text-white">
                  <Zap className="h-4 w-4 text-amber-400" />
                  <span>Свободный ИИ: Groq</span>
                </div>
                {status?.active_ai_preset === "groq" && (
                  <span className="rounded-full bg-emerald-500/20 px-2 py-0.5 text-[11px] font-semibold text-emerald-400 border border-emerald-500/30">
                    АКТИВЕН
                  </span>
                )}
              </div>
              <p className="mt-2 text-xs text-slate-400 leading-relaxed">
                Сверхбыстрые Llama 3.3 70B и DeepSeek R1. Прописывает эндпоинт для всей системы.
              </p>
            </div>

            <button
              onClick={() => handleApplyPreset("groq_ai")}
              disabled={applyingPreset !== null || status?.active_ai_preset === "groq"}
              className={`mt-4 flex w-full items-center justify-center gap-2 rounded-lg py-2.5 text-xs font-bold transition-all ${
                status?.active_ai_preset === "groq"
                  ? "bg-slate-800 text-slate-400 cursor-default"
                  : "bg-amber-500 text-slate-950 hover:bg-amber-400 active:scale-95 shadow-md shadow-amber-500/20"
              }`}
            >
              {status?.active_ai_preset === "groq" ? (
                <>
                  <Check className="h-4 w-4 text-emerald-400" />
                  <span>Groq подключен</span>
                </>
              ) : (
                <>
                  <Zap className="h-4 w-4" />
                  <span>{applyingPreset === "groq_ai" ? "Подключение..." : "Подключить Groq"}</span>
                </>
              )}
            </button>
          </div>

          {/* Card 3: OpenRouter */}
          <div className="flex flex-col justify-between rounded-xl border border-slate-800 bg-slate-900/70 p-5 backdrop-blur transition-all hover:border-slate-700">
            <div>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 font-bold text-white">
                  <Layers className="h-4 w-4 text-indigo-400" />
                  <span>Шлюз: OpenRouter</span>
                </div>
                {status?.active_ai_preset === "openrouter" && (
                  <span className="rounded-full bg-emerald-500/20 px-2 py-0.5 text-[11px] font-semibold text-emerald-400 border border-emerald-500/30">
                    АКТИВЕН
                  </span>
                )}
              </div>
              <p className="mt-2 text-xs text-slate-400 leading-relaxed">
                Доступ к десяткам бесплатных моделей с суффиксом <code>:free</code> (DeepSeek, Llama, Gemini).
              </p>
            </div>

            <button
              onClick={() => handleApplyPreset("openrouter_ai")}
              disabled={applyingPreset !== null || status?.active_ai_preset === "openrouter"}
              className={`mt-4 flex w-full items-center justify-center gap-2 rounded-lg py-2.5 text-xs font-bold transition-all ${
                status?.active_ai_preset === "openrouter"
                  ? "bg-slate-800 text-slate-400 cursor-default"
                  : "bg-indigo-600 text-white hover:bg-indigo-500 active:scale-95 shadow-md shadow-indigo-600/20"
              }`}
            >
              {status?.active_ai_preset === "openrouter" ? (
                <>
                  <Check className="h-4 w-4 text-emerald-400" />
                  <span>OpenRouter подключен</span>
                </>
              ) : (
                <>
                  <Layers className="h-4 w-4" />
                  <span>{applyingPreset === "openrouter_ai" ? "Подключение..." : "Подключить OpenRouter"}</span>
                </>
              )}
            </button>
          </div>
        </div>
      </div>

      {/* COLLAPSIBLE ADVANCED SECTION: Only for those who want URLs and JSON */}
      <div className="rounded-xl border border-slate-800/80 bg-slate-950/40 overflow-hidden">
        <button
          onClick={() => setShowAdvanced(!showAdvanced)}
          className="flex w-full items-center justify-between p-4 text-xs font-medium text-slate-400 hover:text-slate-200 transition-colors"
        >
          <div className="flex items-center gap-2">
            <Server className="h-4 w-4 text-slate-500" />
            <span>Для продвинутых пользователей: Список эндпоинтов и адресов</span>
          </div>
          {showAdvanced ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
        </button>

        {showAdvanced && (
          <div className="border-t border-slate-800/80 p-5 space-y-4 animate-in fade-in-50">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
              {freeHubData?.providers.map((p) => {
                const isCopied = copiedId === p.id;
                return (
                  <div key={p.id} className="rounded-lg border border-slate-800 bg-slate-900/80 p-3">
                    <div className="flex items-center justify-between">
                      <span className="font-semibold text-white">{p.name}</span>
                      <span className="text-[10px] text-slate-400">{p.status_text}</span>
                    </div>
                    <div className="mt-2 flex items-center justify-between gap-2 bg-slate-950 p-2 rounded border border-slate-850">
                      <code className="text-emerald-400 font-mono text-[11px] truncate">{p.base_url}</code>
                      <button
                        onClick={() => handleCopy(p.base_url, p.id)}
                        className="rounded p-1 text-slate-400 hover:text-white"
                        title="Скопировать"
                      >
                        {isCopied ? <Check className="h-3.5 w-3.5 text-emerald-400" /> : <Copy className="h-3.5 w-3.5" />}
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
