import React, { useState, useEffect } from "react";
import {
  Gauge,
  Play,
  RotateCw,
  Zap,
  CheckCircle2,
  XCircle,
  AlertTriangle,
  Clock,
  ExternalLink,
  Sparkles,
  Trophy,
  ArrowRight,
  RefreshCw,
  Server,
  Activity,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { SpeedBenchmarkReport } from "../types";

export const SpeedBenchmarkView: React.FC = () => {
  const [report, setReport] = useState<SpeedBenchmarkReport | null>(null);
  const [isRunning, setIsRunning] = useState<boolean>(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const runBenchmark = async () => {
    setIsRunning(true);
    setErrorMsg(null);
    try {
      const res = await invoke<SpeedBenchmarkReport>("run_ai_speed_benchmark");
      setReport(res);
    } catch (err: any) {
      console.error("Speed benchmark failed:", err);
      setErrorMsg(`Ошибка замера скорости: ${err.toString()}`);
    } finally {
      setIsRunning(false);
    }
  };

  useEffect(() => {
    runBenchmark();
  }, []);

  const getRatingBadge = (rating: string, latency?: number) => {
    switch (rating) {
      case "ULTRA_FAST":
        return (
          <span className="px-2 py-0.5 rounded-full text-xs font-bold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 flex items-center gap-1">
            <Zap className="w-3 h-3 text-emerald-400" />
            {latency ? `${latency} мс` : "Молния"}
          </span>
        );
      case "FAST":
        return (
          <span className="px-2 py-0.5 rounded-full text-xs font-bold bg-sky-500/20 text-sky-300 border border-sky-500/30 flex items-center gap-1">
            <CheckCircle2 className="w-3 h-3 text-sky-400" />
            {latency ? `${latency} мс` : "Быстро"}
          </span>
        );
      case "NORMAL":
        return (
          <span className="px-2 py-0.5 rounded-full text-xs font-bold bg-amber-500/20 text-amber-300 border border-amber-500/30 flex items-center gap-1">
            <Clock className="w-3 h-3 text-amber-400" />
            {latency ? `${latency} мс` : "Средне"}
          </span>
        );
      case "SLOW":
        return (
          <span className="px-2 py-0.5 rounded-full text-xs font-bold bg-orange-500/20 text-orange-300 border border-orange-500/30 flex items-center gap-1">
            <AlertTriangle className="w-3 h-3 text-orange-400" />
            {latency ? `${latency} мс` : "Медленно"}
          </span>
        );
      default:
        return (
          <span className="px-2 py-0.5 rounded-full text-xs font-bold bg-rose-500/20 text-rose-300 border border-rose-500/30 flex items-center gap-1">
            <XCircle className="w-3 h-3 text-rose-400" />
            Заблокирован
          </span>
        );
    }
  };

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 animate-fadeIn">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 bg-surface/50 border border-border/40 p-6 rounded-2xl backdrop-blur-xl">
        <div className="space-y-1">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-xl bg-gradient-to-tr from-amber-500/20 to-orange-500/20 border border-amber-500/30 text-amber-400">
              <Activity className="w-6 h-6" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h1 className="text-2xl font-bold tracking-tight text-white">
                  Спидтест ИИ & Задержка каналов
                </h1>
                <span className="px-2.5 py-0.5 text-xs font-semibold rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/30">
                  Live Benchmark
                </span>
              </div>
              <p className="text-sm text-slate-400">
                Параллельный замер сетевой задержки (RTT / пинг) и проверка блокировок к ведущим ИИ-провайдерам из РФ
              </p>
            </div>
          </div>
        </div>

        <button
          onClick={runBenchmark}
          disabled={isRunning}
          className="flex items-center gap-2.5 px-6 py-3 rounded-xl bg-gradient-to-r from-amber-500 to-orange-600 hover:from-amber-400 hover:to-orange-500 text-slate-950 font-bold text-sm shadow-lg shadow-amber-500/20 active:scale-95 transition-all disabled:opacity-50"
        >
          <RotateCw className={`w-4 h-4 ${isRunning ? "animate-spin" : ""}`} />
          {isRunning ? "Идёт замер скорости..." : "⚡ Запустить тест скорости"}
        </button>
      </div>

      {/* Error Banner */}
      {errorMsg && (
        <div className="p-4 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-sm flex items-center justify-between">
          <span>{errorMsg}</span>
          <button onClick={() => setErrorMsg(null)} className="text-xs underline ml-4">
            Закрыть
          </button>
        </div>
      )}

      {/* Champion Banner */}
      {report?.fastest_provider_name && (
        <div className="p-6 rounded-2xl border border-amber-500/40 bg-gradient-to-r from-amber-950/40 via-surface to-slate-900/60 shadow-lg shadow-amber-500/5">
          <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
            <div className="flex items-center gap-4">
              <div className="p-3 rounded-2xl bg-amber-500/20 text-amber-400 border border-amber-500/30 shrink-0">
                <Trophy className="w-8 h-8" />
              </div>
              <div>
                <span className="text-xs font-bold uppercase tracking-wider text-amber-400 block">
                  Лидер по скорости прямо сейчас
                </span>
                <h3 className="text-xl font-bold text-white mt-0.5">
                  {report.fastest_provider_name}
                </h3>
                <p className="text-xs text-slate-300 mt-1">
                  {report.recommendation_ru}
                </p>
              </div>
            </div>

            <div className="flex items-baseline gap-1.5 bg-amber-500/10 border border-amber-500/30 px-5 py-3 rounded-2xl shrink-0">
              <span className="text-3xl font-extrabold text-amber-400 font-mono">
                {report.fastest_provider_rtt_ms}
              </span>
              <span className="text-xs font-semibold text-amber-300">мс</span>
            </div>
          </div>
        </div>
      )}

      {/* Results Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {report?.results.map((target) => (
          <div
            key={target.id}
            className={`p-5 rounded-xl border transition-all ${
              target.is_available
                ? "bg-surface/50 border-border/50 hover:border-amber-500/40"
                : "bg-surface/30 border-rose-500/20 opacity-75"
            }`}
          >
            <div className="flex items-center justify-between mb-3">
              <span className="text-[10px] font-mono uppercase tracking-wider px-2 py-0.5 rounded bg-surface-elevated text-slate-400 border border-border/40">
                {target.category}
              </span>
              {getRatingBadge(target.speed_rating, target.latency_ms)}
            </div>

            <h4 className="font-bold text-white text-sm mb-1">{target.name}</h4>
            <p className="text-xs text-slate-400 truncate font-mono" title={target.endpoint}>
              {target.endpoint}
            </p>

            <div className="mt-4 pt-3 border-t border-border/30 flex items-center justify-between text-xs">
              <span className="text-slate-300">{target.note_ru}</span>
              <span
                className={`font-mono font-semibold ${
                  target.is_available ? "text-emerald-400" : "text-rose-400"
                }`}
              >
                {target.is_available ? "Доступен" : "Блокировка"}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
