import React, { useState, useEffect } from "react";
import {
  Wrench,
  CheckCircle2,
  AlertTriangle,
  X,
  Play,
  RotateCw,
  Copy,
  Check,
  ShieldCheck,
  Terminal,
  History,
} from "lucide-react";
import { FixPreview, FixExecutionResult } from "../types";
import { invoke } from "@tauri-apps/api/core";

interface SafeFixModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export const SafeFixModal: React.FC<SafeFixModalProps> = ({
  isOpen,
  onClose,
  onSuccess,
}) => {
  const [fixes, setFixes] = useState<FixPreview[]>([]);
  const [selectedFix, setSelectedFix] = useState<FixPreview | null>(null);
  const [isExecuting, setIsExecuting] = useState<boolean>(false);
  const [executionResult, setExecutionResult] = useState<FixExecutionResult | null>(null);
  const [copied, setCopied] = useState<boolean>(false);

  useEffect(() => {
    if (isOpen) {
      loadFixes();
      setExecutionResult(null);
    }
  }, [isOpen]);

  const loadFixes = async () => {
    try {
      const res = await invoke("get_safe_fixes");
      if (res && Array.isArray(res)) {
        setFixes(res as FixPreview[]);
        if (res.length > 0) setSelectedFix(res[0]);
      }
    } catch (e) {
      console.warn("Could not load fixes:", e);
      // Fallback preview
      const fallback: FixPreview[] = [
        {
          fix_id: "flush_dns",
          title_ru: "Очистка локального кэша DNS (Flush DNS)",
          description_ru: "Сбрасывает системный кэш DNS-резолвера Windows. Устраняет застрявшие устаревшие IP-адреса.",
          command_preview: "ipconfig /flushdns",
          subsystem: "DNS",
          is_destructive: false,
          requires_admin: false,
          rollback_preview: "Кэш автоматически обновляется при следующих запросах.",
        },
        {
          fix_id: "clear_stale_proxy_env",
          title_ru: "Удаление устаревших переменных HTTP_PROXY / HTTPS_PROXY",
          description_ru: "Очищает неработающие переменные окружения, устраняя 502 Bad Gateway в терминале.",
          command_preview: "[System.Environment]::SetEnvironmentVariable('HTTP_PROXY', $null, 'User')",
          subsystem: "Environment",
          is_destructive: false,
          requires_admin: false,
          rollback_preview: "Предыдущее значение можно вернуть в любой момент.",
        },
      ];
      setFixes(fallback);
      setSelectedFix(fallback[0]);
    }
  };

  const handleApply = async () => {
    if (!selectedFix) return;
    setIsExecuting(true);
    try {
      const res = await invoke("apply_safe_fix", { fixId: selectedFix.fix_id });
      setExecutionResult(res as FixExecutionResult);
      onSuccess();
    } catch (e: any) {
      console.error("Fix execution failed:", e);
      setExecutionResult({
        success: false,
        snapshot_id: "err",
        message_ru: `Ошибка выполнения: ${e}`,
        stdout: "",
        stderr: String(e),
      });
    } finally {
      setIsExecuting(false);
    }
  };

  const copyCommand = (cmd: string) => {
    navigator.clipboard.writeText(cmd);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-150">
      <div className="relative w-full max-w-2xl rounded-2xl bg-surface border border-surface-border shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
        {/* Modal Header */}
        <div className="p-5 border-b border-surface-border flex items-center justify-between bg-surface-elevated/40">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-primary/10 border border-primary/20">
              <Wrench className="w-5 h-5 text-primary" />
            </div>
            <div>
              <h3 className="text-sm font-bold text-white tracking-tight">
                Безопасный центр устранения неполадок (Safe Fix Engine)
              </h3>
              <p className="text-[11px] text-slate-400">
                Недеструктивные разрешенные операции с подтверждением и откатом
              </p>
            </div>
          </div>

          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-surface-elevated transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Modal Body */}
        <div className="p-6 overflow-y-auto space-y-5">
          {/* Action List Selector */}
          <div className="space-y-2">
            <label className="text-xs font-bold text-slate-300 uppercase tracking-wider">
              Выберите действие для исправления:
            </label>
            <div className="grid grid-cols-1 gap-2">
              {fixes.map((f) => (
                <button
                  key={f.fix_id}
                  onClick={() => {
                    setSelectedFix(f);
                    setExecutionResult(null);
                  }}
                  className={`text-left p-3.5 rounded-xl border transition-all ${
                    selectedFix?.fix_id === f.fix_id
                      ? "bg-primary/15 border-primary/50 text-white shadow-sm"
                      : "bg-surface-elevated/40 border-surface-border text-slate-300 hover:bg-surface-elevated"
                  }`}
                >
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-xs font-bold">{f.title_ru}</span>
                    <span className="text-[10px] font-mono uppercase px-2 py-0.5 rounded bg-surface border border-surface-border text-slate-400">
                      {f.subsystem}
                    </span>
                  </div>
                  <p className="text-[11px] text-slate-400 line-clamp-2">{f.description_ru}</p>
                </button>
              ))}
            </div>
          </div>

          {/* Dry Run / Preview Box */}
          {selectedFix && (
            <div className="p-4 rounded-xl bg-slate-950/70 border border-surface-border space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-[11px] font-bold uppercase tracking-wider text-primary flex items-center gap-1.5">
                  <Terminal className="w-3.5 h-3.5" />
                  Предварительный просмотр команды (Dry-Run):
                </span>
                <button
                  onClick={() => copyCommand(selectedFix.command_preview)}
                  className="flex items-center gap-1 text-[11px] text-slate-400 hover:text-white transition-colors"
                >
                  {copied ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
                  <span>{copied ? "Скопировано" : "Копировать"}</span>
                </button>
              </div>

              <div className="p-2.5 rounded-lg bg-black/60 border border-white/5 font-mono text-xs text-emerald-300 break-all select-text">
                {selectedFix.command_preview}
              </div>

              <div className="flex items-center gap-2 text-[11px] text-slate-400 pt-1">
                <ShieldCheck className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                <span>Откат: {selectedFix.rollback_preview}</span>
              </div>
            </div>
          )}

          {/* Execution Result Banner */}
          {executionResult && (
            <div
              className={`p-4 rounded-xl border space-y-2 animate-in fade-in duration-200 ${
                executionResult.success
                  ? "bg-emerald-950/30 border-emerald-500/40 text-emerald-200"
                  : "bg-rose-950/30 border-rose-500/40 text-rose-200"
              }`}
            >
              <div className="flex items-center gap-2 text-xs font-bold">
                {executionResult.success ? (
                  <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                ) : (
                  <AlertTriangle className="w-4 h-4 text-rose-400" />
                )}
                <span>{executionResult.message_ru}</span>
              </div>
              {executionResult.stdout && (
                <pre className="text-[10px] font-mono bg-black/40 p-2 rounded text-slate-300 overflow-x-auto">
                  {executionResult.stdout}
                </pre>
              )}
            </div>
          )}
        </div>

        {/* Modal Footer */}
        <div className="p-4 border-t border-surface-border bg-surface-elevated/40 flex items-center justify-between">
          <span className="text-[11px] text-slate-400 font-mono">
            Требуется явное подтверждение
          </span>

          <div className="flex items-center gap-2">
            <button
              onClick={onClose}
              className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-300 hover:text-white hover:bg-surface-elevated transition-colors"
            >
              Закрыть
            </button>

            <button
              onClick={handleApply}
              disabled={isExecuting || !selectedFix}
              className={`flex items-center gap-2 px-5 py-2 rounded-xl text-xs font-semibold shadow-lg transition-all active:scale-95 ${
                isExecuting
                  ? "bg-slate-800 text-slate-500 cursor-not-allowed"
                  : "bg-primary text-slate-950 hover:bg-primary-hover shadow-primary/20"
              }`}
            >
              {isExecuting ? (
                <>
                  <RotateCw className="w-3.5 h-3.5 animate-spin" />
                  <span>Выполнение...</span>
                </>
              ) : (
                <>
                  <Play className="w-3.5 h-3.5 fill-current" />
                  <span>Применить исправление</span>
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
