import React, { useState, useEffect } from "react";
import {
  Globe,
  Zap,
  Cpu,
  Shield,
  Copy,
  Check,
  RefreshCw,
  ExternalLink,
  Server,
  Layers,
  Sparkles,
  Info,
  CheckCircle2,
  AlertTriangle,
  XCircle,
} from "lucide-react";
import { FreeHubReport, FreeProviderInfo, PublicDohInfo } from "../types";
import { invoke } from "@tauri-apps/api/core";

export const FreeHubView: React.FC = () => {
  const [data, setData] = useState<FreeHubReport | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [activeCategory, setActiveCategory] = useState<string>("all");
  const [activeIdeTab, setActiveIdeTab] = useState<"cursor" | "continue" | "zed">("cursor");

  const loadData = async () => {
    setIsLoading(true);
    try {
      const res = await invoke<FreeHubReport>("get_free_hub");
      setData(res);
    } catch (e) {
      console.error("Failed to load Free Hub data:", e);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleCopy = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => {
      setCopiedId(null);
    }, 2000);
  };

  const filteredProviders = data?.providers.filter((p) => {
    if (activeCategory === "all") return true;
    if (activeCategory === "cloud") return p.category.includes("Cloud") || p.category.includes("API");
    if (activeCategory === "router") return p.category.includes("шлюз") || p.category.includes("экосистема");
    if (activeCategory === "local") return p.category.includes("Локальный");
    return true;
  });

  return (
    <div className="space-y-6">
      {/* Top Banner */}
      <div className="rounded-xl border border-sky-500/30 bg-gradient-to-r from-sky-950/40 via-slate-900/60 to-slate-900/40 p-6 backdrop-blur">
        <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div>
            <div className="flex items-center gap-2">
              <span className="flex h-6 w-6 items-center justify-center rounded-lg bg-sky-500/20 text-sky-400">
                <Sparkles className="h-4 w-4" />
              </span>
              <h2 className="text-xl font-bold tracking-tight text-white">
                Каталог свободного доступа & Зеркала ИИ
              </h2>
              <span className="rounded-full bg-emerald-500/20 px-2.5 py-0.5 text-xs font-semibold text-emerald-400 border border-emerald-500/30">
                Free & Open Access
              </span>
            </div>
            <p className="mt-1 text-sm text-slate-400">
              Бесплатные облачные API, открытые провайдеры (Groq, OpenRouter, DeepSeek), локальные нейросети и защищенные DoH-резолверы для разработчиков.
            </p>
          </div>

          <button
            onClick={loadData}
            disabled={isLoading}
            className="flex items-center gap-2 self-start rounded-lg border border-sky-500/40 bg-sky-500/10 px-4 py-2 text-sm font-medium text-sky-300 transition-all hover:bg-sky-500/20 active:scale-95 disabled:opacity-50"
          >
            <RefreshCw className={`h-4 w-4 ${isLoading ? "animate-spin text-sky-400" : ""}`} />
            {isLoading ? "Тестирование эндпоинтов..." : "Проверить доступность"}
          </button>
        </div>

        {/* Quick Highlights */}
        <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-3 border-t border-slate-800/80 pt-4 text-xs text-slate-300">
          <div className="flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-emerald-400"></span>
            <span><b>Groq & OpenRouter:</b> Бесплатные модели Llama 3.3 & DeepSeek R1</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-sky-400"></span>
            <span><b>DoH (DNS-over-HTTPS):</b> Защита от блокировок и подмены DNS провайдером</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-amber-400"></span>
            <span><b>Ollama / LM Studio:</b> 100% локально на ПК без интернета и ключей</span>
          </div>
        </div>
      </div>

      {/* Category Pills */}
      <div className="flex flex-wrap items-center gap-2 border-b border-slate-800 pb-3">
        <button
          onClick={() => setActiveCategory("all")}
          className={`rounded-lg px-3 py-1.5 text-xs font-medium transition-all ${
            activeCategory === "all"
              ? "bg-sky-500 text-white shadow-lg shadow-sky-500/20"
              : "bg-slate-900/80 text-slate-400 hover:bg-slate-800 hover:text-slate-200"
          }`}
        >
          Все свободные провайдеры ({data?.providers.length || 0})
        </button>
        <button
          onClick={() => setActiveCategory("cloud")}
          className={`flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-medium transition-all ${
            activeCategory === "cloud"
              ? "bg-sky-500 text-white shadow-lg shadow-sky-500/20"
              : "bg-slate-900/80 text-slate-400 hover:bg-slate-800 hover:text-slate-200"
          }`}
        >
          <Zap className="h-3.5 w-3.5" />
          Бесплатные Cloud API (Groq, DeepSeek, Mistral)
        </button>
        <button
          onClick={() => setActiveCategory("router")}
          className={`flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-medium transition-all ${
            activeCategory === "router"
              ? "bg-sky-500 text-white shadow-lg shadow-sky-500/20"
              : "bg-slate-900/80 text-slate-400 hover:bg-slate-800 hover:text-slate-200"
          }`}
        >
          <Layers className="h-3.5 w-3.5" />
          Шлюзы и экосистемы (OpenRouter, HuggingFace)
        </button>
        <button
          onClick={() => setActiveCategory("local")}
          className={`flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-medium transition-all ${
            activeCategory === "local"
              ? "bg-sky-500 text-white shadow-lg shadow-sky-500/20"
              : "bg-slate-900/80 text-slate-400 hover:bg-slate-800 hover:text-slate-200"
          }`}
        >
          <Cpu className="h-3.5 w-3.5" />
          Локальный ИИ на ПК (Ollama, LM Studio)
        </button>
      </div>

      {/* Providers Grid */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        {filteredProviders?.map((p) => {
          const isCopied = copiedId === p.id;
          const statusBg = p.is_accessible
            ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-400"
            : "border-slate-700/50 bg-slate-800/40 text-slate-400";

          return (
            <div
              key={p.id}
              className="flex flex-col justify-between rounded-xl border border-slate-800 bg-slate-900/70 p-5 backdrop-blur transition-all hover:border-slate-700"
            >
              <div>
                {/* Header */}
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <div className="flex items-center gap-2">
                      <h3 className="font-semibold text-white">{p.name}</h3>
                      <span className="rounded bg-slate-800 px-2 py-0.5 text-[10px] font-medium text-slate-300">
                        {p.category}
                      </span>
                    </div>
                    <p className="mt-1 text-xs text-slate-400 leading-relaxed">
                      {p.description_ru}
                    </p>
                  </div>

                  {/* Status Badge */}
                  <div
                    className={`flex items-center gap-1.5 whitespace-nowrap rounded-lg border px-2.5 py-1 text-xs font-medium ${statusBg}`}
                  >
                    {p.is_accessible ? (
                      <CheckCircle2 className="h-3.5 w-3.5 text-emerald-400" />
                    ) : (
                      <AlertTriangle className="h-3.5 w-3.5 text-amber-400" />
                    )}
                    <span>
                      {p.direct_rtt_ms !== null && p.direct_rtt_ms !== undefined
                        ? `${p.direct_rtt_ms} ms`
                        : "Не запущен"}
                    </span>
                  </div>
                </div>

                {/* Popular Models */}
                <div className="mt-3">
                  <span className="text-[11px] font-medium text-slate-400">Популярные бесплатные модели:</span>
                  <div className="mt-1 flex flex-wrap gap-1.5">
                    {p.popular_models.map((m) => (
                      <span
                        key={m}
                        className="rounded-md border border-slate-700/80 bg-slate-800/70 px-2 py-0.5 font-mono text-[11px] text-sky-300"
                      >
                        {m}
                      </span>
                    ))}
                  </div>
                </div>

                {/* Base URL Copy Bar */}
                <div className="mt-3.5 rounded-lg border border-slate-800 bg-slate-950/80 p-2.5">
                  <div className="flex items-center justify-between text-[11px]">
                    <span className="font-mono text-slate-400">Base URL для IDE:</span>
                    <span className="text-slate-400">{p.auth_note}</span>
                  </div>
                  <div className="mt-1.5 flex items-center justify-between gap-2">
                    <code className="truncate font-mono text-xs text-emerald-400">
                      {p.base_url}
                    </code>
                    <button
                      onClick={() => handleCopy(p.base_url, p.id)}
                      className={`flex items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium transition-all ${
                        isCopied
                          ? "bg-emerald-500/20 text-emerald-300 border border-emerald-500/30"
                          : "bg-slate-800 text-slate-300 hover:bg-slate-700"
                      }`}
                      title="Скопировать Base URL для вставки в IDE"
                    >
                      {isCopied ? (
                        <>
                          <Check className="h-3.5 w-3.5" />
                          <span>Скопировано!</span>
                        </>
                      ) : (
                        <>
                          <Copy className="h-3.5 w-3.5" />
                          <span>Скопировать</span>
                        </>
                      )}
                    </button>
                  </div>
                </div>
              </div>

              {/* Card Footer */}
              <div className="mt-4 flex items-center justify-between border-t border-slate-800/80 pt-3 text-xs text-slate-400">
                <span className="truncate">Рекомендован: {p.recommended_for}</span>
                <a
                  href={p.docs_url}
                  target="_blank"
                  rel="noreferrer"
                  className="flex items-center gap-1 font-medium text-sky-400 hover:text-sky-300"
                >
                  <span>Доки/Ключ</span>
                  <ExternalLink className="h-3 w-3" />
                </a>
              </div>
            </div>
          );
        })}
      </div>

      {/* DoH Resolvers Section */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-6 backdrop-blur">
        <div className="flex items-center gap-2 border-b border-slate-800 pb-4">
          <Shield className="h-5 w-5 text-sky-400" />
          <div>
            <h3 className="font-bold text-white">
              Публичные DoH-резолверы (DNS-over-HTTPS)
            </h3>
            <p className="text-xs text-slate-400">
              Защищают от подмены DNS провайдером и DNS-блокировок. Запросы шифруются через порт HTTPS (443).
            </p>
          </div>
        </div>

        <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
          {data?.doh_resolvers.map((d) => {
            const isCopied = copiedId === d.doh_url;
            return (
              <div
                key={d.name}
                className="flex flex-col justify-between rounded-lg border border-slate-800 bg-slate-950/60 p-4 transition-all hover:border-slate-700"
              >
                <div>
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-white">{d.name}</span>
                    <span
                      className={`h-2 w-2 rounded-full ${
                        d.is_tested_ok ? "bg-emerald-400" : "bg-slate-600"
                      }`}
                    ></span>
                  </div>
                  <p className="mt-1 text-xs text-slate-400">{d.description_ru}</p>
                  <div className="mt-2 font-mono text-[11px] text-slate-300">
                    <div>IP 1: {d.primary_ip}</div>
                    <div>IP 2: {d.secondary_ip}</div>
                  </div>
                </div>

                <div className="mt-3 pt-3 border-t border-slate-800/80">
                  <div className="truncate font-mono text-[11px] text-sky-300">
                    {d.doh_url}
                  </div>
                  <button
                    onClick={() => handleCopy(d.doh_url, d.doh_url)}
                    className={`mt-2 flex w-full items-center justify-center gap-1.5 rounded py-1 text-xs font-medium transition-all ${
                      isCopied
                        ? "bg-emerald-500/20 text-emerald-300 border border-emerald-500/30"
                        : "bg-slate-800 text-slate-300 hover:bg-slate-700"
                    }`}
                  >
                    {isCopied ? <Check className="h-3 w-3" /> : <Copy className="h-3 w-3" />}
                    {isCopied ? "DoH URL скопирован" : "Скопировать DoH URL"}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Quick Setup Instructions for IDEs */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-6 backdrop-blur">
        <div className="flex items-center justify-between border-b border-slate-800 pb-3">
          <div className="flex items-center gap-2">
            <Server className="h-5 w-5 text-indigo-400" />
            <h3 className="font-bold text-white">Как подключить свободные модели в вашу IDE</h3>
          </div>
          <div className="flex items-center gap-1.5">
            <button
              onClick={() => setActiveIdeTab("cursor")}
              className={`rounded px-2.5 py-1 text-xs font-medium transition-all ${
                activeIdeTab === "cursor" ? "bg-indigo-600 text-white" : "text-slate-400 hover:bg-slate-800"
              }`}
            >
              Cursor
            </button>
            <button
              onClick={() => setActiveIdeTab("continue")}
              className={`rounded px-2.5 py-1 text-xs font-medium transition-all ${
                activeIdeTab === "continue" ? "bg-indigo-600 text-white" : "text-slate-400 hover:bg-slate-800"
              }`}
            >
              VS Code (Continue / Cline)
            </button>
            <button
              onClick={() => setActiveIdeTab("zed")}
              className={`rounded px-2.5 py-1 text-xs font-medium transition-all ${
                activeIdeTab === "zed" ? "bg-indigo-600 text-white" : "text-slate-400 hover:bg-slate-800"
              }`}
            >
              Zed Editor
            </button>
          </div>
        </div>

        <div className="mt-4 text-xs text-slate-300">
          {activeIdeTab === "cursor" && (
            <div className="space-y-2">
              <p><b>В Cursor:</b> Откройте <code>Settings (Ctrl + Shift + J)</code> → вкладка <b>Models</b>:</p>
              <ol className="list-decimal space-y-1 pl-5 text-slate-400">
                <li>В разделе <b>OpenAI API Key</b> укажите <code>Override OpenAI Base URL</code>: <code className="text-emerald-300">https://api.groq.com/openai/v1</code> или <code className="text-emerald-300">https://openrouter.ai/api/v1</code>.</li>
                <li>Вставьте бесплатный API-ключ от выбранного сервиса.</li>
                <li>Добавьте имя модели в список (например, <code>llama-3.3-70b-versatile</code> или <code>deepseek/deepseek-r1:free</code>).</li>
              </ol>
            </div>
          )}

          {activeIdeTab === "continue" && (
            <div className="space-y-2">
              <p><b>В расширении Continue / Cline:</b> Отредактируйте файл <code>~/.continue/config.json</code>:</p>
              <pre className="overflow-x-auto rounded-lg bg-slate-950 p-3 font-mono text-[11px] text-sky-300">
{`{
  "models": [
    {
      "title": "Groq Llama 3.3 70B (Free)",
      "provider": "openai",
      "model": "llama-3.3-70b-versatile",
      "apiBase": "https://api.groq.com/openai/v1",
      "apiKey": "<ВАШ_БЕСПЛАТНЫЙ_GROQ_КЛЮЧ>"
    },
    {
      "title": "Ollama Local (Offline)",
      "provider": "ollama",
      "model": "qwen2.5-coder:7b",
      "apiBase": "http://localhost:11434"
    }
  ]
}`}
              </pre>
            </div>
          )}

          {activeIdeTab === "zed" && (
            <div className="space-y-2">
              <p><b>В Zed Editor:</b> Откройте <code>settings.json</code>:</p>
              <pre className="overflow-x-auto rounded-lg bg-slate-950 p-3 font-mono text-[11px] text-sky-300">
{`{
  "language_models": {
    "openai": {
      "api_url": "https://api.groq.com/openai/v1",
      "available_models": ["llama-3.3-70b-versatile"]
    }
  }
}`}
              </pre>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
