import React, { useState } from "react";
import {
  KeyRound,
  ShieldCheck,
  ShieldAlert,
  Zap,
  Globe,
  Clock,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  Copy,
  Check,
  Eye,
  EyeOff,
  Sparkles,
  RefreshCw,
  Layers,
} from "lucide-react";
import { ApiKeyValidationReport } from "../types";
import { invoke } from "@tauri-apps/api/core";

interface ProviderOption {
  id: string;
  name: string;
  placeholder: string;
  description: string;
  badge: string;
  defaultModels: string;
}

const PROVIDERS: ProviderOption[] = [
  {
    id: "gemini",
    name: "Google Gemini API",
    placeholder: "AIzaSy...",
    description: "Бесплатные и платные ключи Google AI Studio (Gemini 2.5 Pro / Flash)",
    badge: "Google",
    defaultModels: "gemini-2.5-pro, gemini-2.5-flash",
  },
  {
    id: "openai",
    name: "OpenAI / ChatGPT",
    placeholder: "sk-proj-...",
    description: "API-ключи OpenAI платформы (GPT-4o, o3-mini, o1-preview)",
    badge: "OpenAI",
    defaultModels: "gpt-4o, o3-mini, o1",
  },
  {
    id: "anthropic",
    name: "Anthropic Claude",
    placeholder: "sk-ant-api03-...",
    description: "Прямой API доступ к моделям Claude 3.7 Sonnet / Haiku",
    badge: "Anthropic",
    defaultModels: "claude-3-7-sonnet, claude-3-5-haiku",
  },
  {
    id: "groq",
    name: "Groq LPU",
    placeholder: "gsk_...",
    description: "Ультра-быстрый инференс с 500+ токенов/сек (Llama 3.3, Mixtral)",
    badge: "Groq",
    defaultModels: "llama-3.3-70b, deepseek-r1-distill",
  },
  {
    id: "openrouter",
    name: "OpenRouter",
    placeholder: "sk-or-v1-...",
    description: "Универсальный агрегатор всех ИИ-моделей в одном балансе",
    badge: "Gateway",
    defaultModels: "Все ведущие модели",
  },
];

export const ApiKeyInspectorView: React.FC = () => {
  const [selectedProvider, setSelectedProvider] = useState<string>("gemini");
  const [apiKey, setApiKey] = useState<string>("");
  const [showKey, setShowKey] = useState<boolean>(false);
  const [useProxy, setUseProxy] = useState<boolean>(true);
  const [proxyUrl, setProxyUrl] = useState<string>("http://127.0.0.1:53200");
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [report, setReport] = useState<ApiKeyValidationReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState<boolean>(false);

  const activeProvider = PROVIDERS.find((p) => p.id === selectedProvider) || PROVIDERS[0];

  const handleValidate = async () => {
    if (!apiKey.trim()) {
      setError("Пожалуйста, введите API-ключ для проверки.");
      return;
    }

    setIsLoading(true);
    setError(null);
    setReport(null);

    try {
      const effectiveProxy = useProxy ? proxyUrl : undefined;
      const res = await invoke<ApiKeyValidationReport>("validate_ai_api_key", {
        provider: selectedProvider,
        apiKey: apiKey.trim(),
        proxyUrl: effectiveProxy,
      });
      setReport(res);
    } catch (err: any) {
      setError(typeof err === "string" ? err : JSON.stringify(err));
    } finally {
      setIsLoading(false);
    }
  };

  const copyResult = () => {
    if (!report) return;
    const text = `🔑 Проверка ключа ${report.provider}:
Статус: ${report.is_valid ? "Валиден" : "Недействителен"}
Задержка: ${report.latency_ms} мс
Блокировка GeoIP: ${report.is_geo_blocked ? "ДА (заблокирован в РФ)" : "НЕТ (доступен)"}
${report.quota_info ? `Квота: ${report.quota_info}` : ""}
${report.message}`;

    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="space-y-6">
      {/* Header Banner */}
      <div className="rounded-2xl bg-surface border border-surface-border p-6 shadow-xl relative overflow-hidden">
        <div className="absolute top-0 right-0 -mt-10 -mr-10 w-64 h-64 bg-amber-500/10 rounded-full blur-3xl pointer-events-none" />

        <div className="relative z-10 flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <div className="flex items-center gap-2 mb-1.5">
              <span className="text-[11px] font-semibold uppercase tracking-wider px-2 py-0.5 rounded bg-amber-500/15 text-amber-300 border border-amber-500/25">
                AI API Inspector & Quota
              </span>
              <span className="text-xs text-slate-400 font-mono">
                100% Local-First • Без сохранения ключей на диск
              </span>
            </div>
            <h2 className="text-xl font-bold text-white tracking-tight">
              Инспектор валидности, пинга и квот API-ключей
            </h2>
            <p className="text-xs text-slate-400 mt-1 max-w-2xl leading-relaxed">
              Мгновенно проверяет работоспособность ключа, измеряет задержку подключения (RTT),
              выявляет скрытые региональные блокировки (GeoIP) и отображает доступные модели.
            </p>
          </div>
        </div>
      </div>

      {/* Main Validation Card */}
      <div className="rounded-2xl bg-surface border border-surface-border p-6 space-y-6">
        {/* Provider Selector Chips */}
        <div>
          <label className="block text-xs font-semibold text-slate-300 uppercase tracking-wider mb-2.5">
            Выберите провайдера ИИ
          </label>
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-2.5">
            {PROVIDERS.map((p) => {
              const isSelected = p.id === selectedProvider;
              return (
                <button
                  key={p.id}
                  onClick={() => {
                    setSelectedProvider(p.id);
                    setReport(null);
                    setError(null);
                  }}
                  className={`p-3 rounded-xl border text-left transition-all relative ${
                    isSelected
                      ? "bg-primary/15 border-primary/40 shadow-sm"
                      : "bg-surface-elevated/40 border-border/40 hover:bg-surface-elevated hover:border-border/80"
                  }`}
                >
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-xs font-bold text-white">{p.name}</span>
                  </div>
                  <span className="text-[10px] text-slate-400 line-clamp-1">{p.badge}</span>
                </button>
              );
            })}
          </div>
        </div>

        {/* Key Input Section */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <label className="text-xs font-semibold text-slate-300 uppercase tracking-wider flex items-center gap-2">
              <KeyRound className="w-3.5 h-3.5 text-amber-400" />
              API-ключ ({activeProvider.name})
            </label>
            <span className="text-[11px] text-slate-400">
              {activeProvider.description}
            </span>
          </div>

          <div className="relative">
            <input
              type={showKey ? "text" : "password"}
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder={activeProvider.placeholder}
              className="w-full bg-slate-900/80 border border-slate-700/80 rounded-xl px-4 py-3 text-sm font-mono text-white placeholder-slate-500 focus:outline-none focus:border-primary pr-24"
            />
            <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
              <button
                type="button"
                onClick={() => setShowKey(!showKey)}
                className="p-1.5 text-slate-400 hover:text-white rounded-lg transition-colors"
                title={showKey ? "Скрыть ключ" : "Показать ключ"}
              >
                {showKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
              </button>
              {apiKey && (
                <button
                  type="button"
                  onClick={() => setApiKey("")}
                  className="p-1 text-slate-500 hover:text-rose-400 text-xs rounded transition-colors"
                >
                  Очистить
                </button>
              )}
            </div>
          </div>
        </div>

        {/* Proxy Routing Toggle */}
        <div className="p-4 bg-slate-900/40 rounded-xl border border-slate-800 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <Globe className="w-4 h-4 text-sky-400" />
            <div>
              <span className="text-xs font-semibold text-white">Тестировать через прокси / мост</span>
              <p className="text-[11px] text-slate-400">
                Позволяет проверить, обходит ли ключ региональную блокировку через шлюз
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <input
              type="text"
              value={proxyUrl}
              onChange={(e) => setProxyUrl(e.target.value)}
              disabled={!useProxy}
              className="bg-slate-900 border border-slate-700 rounded-lg px-2.5 py-1 text-xs font-mono text-slate-200 w-44 disabled:opacity-40"
            />
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={useProxy}
                onChange={(e) => setUseProxy(e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-9 h-5 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-primary"></div>
            </label>
          </div>
        </div>

        {/* Action Button */}
        <div className="flex items-center gap-3">
          <button
            onClick={handleValidate}
            disabled={isLoading || !apiKey.trim()}
            className="w-full sm:w-auto px-6 py-3 rounded-xl bg-primary hover:bg-primary/90 text-white font-semibold text-xs active:scale-95 transition-all flex items-center justify-center gap-2 disabled:opacity-50 shadow-lg shadow-primary/20"
          >
            {isLoading ? (
              <>
                <RefreshCw className="w-4 h-4 animate-spin" />
                <span>Проверка соединения с {activeProvider.name}...</span>
              </>
            ) : (
              <>
                <Sparkles className="w-4 h-4" />
                <span>Проверить ключ и замерить пинг</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Error Message */}
      {error && (
        <div className="p-4 bg-rose-500/10 border border-rose-500/30 rounded-2xl flex items-start gap-3">
          <XCircle className="w-5 h-5 text-rose-400 shrink-0 mt-0.5" />
          <div className="text-xs text-rose-200">
            <span className="font-bold">Ошибка тестирования: </span>
            {error}
          </div>
        </div>
      )}

      {/* Report Results */}
      {report && (
        <div className="rounded-2xl bg-surface border border-surface-border p-6 space-y-5 shadow-xl animate-in fade-in duration-200">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-border/40 pb-4">
            <div className="flex items-center gap-3">
              {report.is_valid && !report.is_geo_blocked ? (
                <div className="w-10 h-10 rounded-xl bg-emerald-500/15 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
                  <CheckCircle2 className="w-5 h-5" />
                </div>
              ) : report.is_geo_blocked ? (
                <div className="w-10 h-10 rounded-xl bg-amber-500/15 border border-amber-500/30 flex items-center justify-center text-amber-400">
                  <Globe className="w-5 h-5" />
                </div>
              ) : (
                <div className="w-10 h-10 rounded-xl bg-rose-500/15 border border-rose-500/30 flex items-center justify-center text-rose-400">
                  <AlertTriangle className="w-5 h-5" />
                </div>
              )}

              <div>
                <h3 className="text-base font-bold text-white flex items-center gap-2">
                  {report.provider}
                  <span
                    className={`text-[10px] uppercase font-bold px-2 py-0.5 rounded ${
                      report.is_valid && !report.is_geo_blocked
                        ? "bg-emerald-500/20 text-emerald-300 border border-emerald-500/30"
                        : report.is_geo_blocked
                        ? "bg-amber-500/20 text-amber-300 border border-amber-500/30"
                        : "bg-rose-500/20 text-rose-300 border border-rose-500/30"
                    }`}
                  >
                    {report.is_valid && !report.is_geo_blocked
                      ? "АКТИВЕН И ГОТОВ"
                      : report.is_geo_blocked
                      ? "GEO-BLOCKED В РФ"
                      : "ОШИБКА КЛЮЧА"}
                  </span>
                </h3>
                <p className="text-xs text-slate-300 mt-0.5">{report.message}</p>
              </div>
            </div>

            <button
              onClick={copyResult}
              className="px-3 py-1.5 rounded-lg bg-surface-elevated hover:bg-slate-700/60 border border-border/60 text-slate-300 text-xs flex items-center gap-1.5 self-start sm:self-auto transition-colors"
            >
              {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
              <span>{copied ? "Скопировано!" : "Скопировать отчет"}</span>
            </button>
          </div>

          {/* Metrics Grid */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
            <div className="p-3 bg-slate-900/60 rounded-xl border border-slate-800">
              <span className="text-[10px] text-slate-400 uppercase tracking-wider block mb-1">
                Сетевой отклик (RTT)
              </span>
              <div className="flex items-center gap-1.5">
                <Clock className="w-4 h-4 text-sky-400" />
                <span className="text-base font-bold text-sky-300 font-mono">
                  {report.latency_ms} мс
                </span>
              </div>
            </div>

            <div className="p-3 bg-slate-900/60 rounded-xl border border-slate-800">
              <span className="text-[10px] text-slate-400 uppercase tracking-wider block mb-1">
                HTTP Код
              </span>
              <span className="text-base font-bold text-white font-mono">
                {report.http_status}
              </span>
            </div>

            <div className="p-3 bg-slate-900/60 rounded-xl border border-slate-800">
              <span className="text-[10px] text-slate-400 uppercase tracking-wider block mb-1">
                GeoIP статус
              </span>
              <span
                className={`text-xs font-bold ${
                  report.is_geo_blocked ? "text-amber-400" : "text-emerald-400"
                }`}
              >
                {report.is_geo_blocked ? "Заблокирован в РФ" : "Разрешен"}
              </span>
            </div>

            <div className="p-3 bg-slate-900/60 rounded-xl border border-slate-800">
              <span className="text-[10px] text-slate-400 uppercase tracking-wider block mb-1">
                Квота / Баланс
              </span>
              <span className="text-xs font-semibold text-slate-200 truncate block">
                {report.quota_info || "Стандартный тариф"}
              </span>
            </div>
          </div>

          {/* Available Models */}
          {report.available_models && report.available_models.length > 0 && (
            <div>
              <span className="text-xs font-semibold text-slate-300 uppercase tracking-wider block mb-2">
                Доступные модели на этом ключе ({report.available_models.length}):
              </span>
              <div className="flex flex-wrap gap-1.5">
                {report.available_models.map((m) => (
                  <span
                    key={m}
                    className="px-2.5 py-1 rounded-lg bg-surface-elevated border border-border/60 text-xs font-mono text-slate-200"
                  >
                    {m}
                  </span>
                ))}
              </div>
            </div>
          )}

          {/* Quick GeoIP Advice */}
          {report.is_geo_blocked && (
            <div className="p-4 bg-amber-500/10 border border-amber-500/30 rounded-xl space-y-2">
              <div className="flex items-center gap-2">
                <Globe className="w-4 h-4 text-amber-400" />
                <span className="text-xs font-bold text-amber-300">
                  Как заставить этот ключ работать в РФ:
                </span>
              </div>
              <p className="text-xs text-slate-300 leading-relaxed">
                Перейдите во вкладку <b>«Antigravity Doctor»</b> и включите <b>«Comss.one SmartDNS»</b> или
                <b>«Европейский мост (53200)»</b>. После этого запросы через этот ключ будут автоматически
                проходить с европейских IP без ошибки <i>User location is not supported</i>.
              </p>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
