import React from "react";
import {
  AlertOctagon,
  Terminal,
  Copy,
  Check,
  FileCode,
  ShieldAlert,
  HelpCircle,
  ExternalLink,
} from "lucide-react";
import { FullDiagnosticReport } from "../types";

interface ErrorAnalysisViewProps {
  report: FullDiagnosticReport | null;
}

export const ErrorAnalysisView: React.FC<ErrorAnalysisViewProps> = ({ report }) => {
  const [copiedIndex, setCopiedIndex] = React.useState<number | null>(null);

  const copyCommand = (cmd: string, idx: number) => {
    navigator.clipboard.writeText(cmd);
    setCopiedIndex(idx);
    setTimeout(() => setCopiedIndex(null), 2000);
  };

  if (!report) {
    return (
      <div className="p-12 text-center border border-dashed border-surface-border rounded-2xl bg-surface/50">
        <AlertOctagon className="w-10 h-10 text-slate-500 mx-auto mb-3" />
        <h3 className="text-sm font-bold text-white">Нет данных для анализа</h3>
        <p className="text-xs text-slate-400 mt-1">
          Выполните диагностику для формирования доказательной базы и технических логов.
        </p>
      </div>
    );
  }

  const { verdict } = report;

  // Useful preset recovery scripts for common developer pain points
  const fixRecipes = [
    {
      title: "Синхронизация HTTP_PROXY с действующим портом прокси",
      desc: "Устанавливает переменные окружения пользователя на рабочий порт локального клиента (например, 10809).",
      cmd: `[System.Environment]::SetEnvironmentVariable('HTTP_PROXY', 'http://127.0.0.1:10809', 'User')\n[System.Environment]::SetEnvironmentVariable('HTTPS_PROXY', 'http://127.0.0.1:10809', 'User')`,
    },
    {
      title: "Сброс переменных HTTP_PROXY / HTTPS_PROXY",
      desc: "Очищает переменные окружения, если прокси больше не запущен или вызывает 502 Bad Gateway.",
      cmd: `[System.Environment]::SetEnvironmentVariable('HTTP_PROXY', $null, 'User')\n[System.Environment]::SetEnvironmentVariable('HTTPS_PROXY', $null, 'User')`,
    },
    {
      title: "Очистка локального кэша сокетов и DNS в Windows",
      desc: "Сбрасывает системную таблицу кэша DNS для обновления IP-адресов эндпоинтов ИИ.",
      cmd: `ipconfig /flushdns`,
    },
    {
      title: "Настройка прокси для Git CLI",
      desc: "Позволяет Git обращаться к GitHub и клонировать репозитории через локальный прокси.",
      cmd: `git config --global http.proxy http://127.0.0.1:10809\ngit config --global https.proxy http://127.0.0.1:10809`,
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-bold text-white tracking-tight">
          Анализ корневых причин и доказательная база
        </h2>
        <p className="text-xs text-slate-400 mt-0.5">
          Фактические свидетельства сетевых сбоев, сопоставленные с правилами экспертной системы.
        </p>
      </div>

      {/* Main Issue Card */}
      <div className="p-5 rounded-2xl bg-surface border border-surface-border space-y-4">
        <div className="flex items-start justify-between pb-3 border-b border-surface-border">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-xl bg-primary/10 border border-primary/20 text-primary">
              <AlertOctagon className="w-5 h-5" />
            </div>
            <div>
              <span className="text-[10px] font-mono uppercase text-slate-400">
                Код вердикта: {verdict.primary_issue_code}
              </span>
              <h3 className="text-sm font-bold text-white">{verdict.title_ru}</h3>
            </div>
          </div>
          <span className="text-xs font-mono px-2.5 py-1 rounded-full bg-primary/10 text-primary font-bold border border-primary/30">
            Достоверность: {verdict.confidence_percent}%
          </span>
        </div>

        <div className="text-xs text-slate-300 leading-relaxed">
          {verdict.explanation_ru}
        </div>
      </div>

      {/* Technical Evidence Log */}
      <div className="space-y-2">
        <h3 className="text-xs font-bold uppercase tracking-wider text-slate-400 flex items-center gap-1.5">
          <FileCode className="w-3.5 h-3.5 text-primary" />
          <span>Технические свидетельства (Evidence Trace)</span>
        </h3>

        <div className="p-4 rounded-xl bg-slate-950 border border-surface-border font-mono text-xs text-slate-300 space-y-1.5 overflow-x-auto">
          {verdict.technical_evidence.map((ev, idx) => (
            <div key={idx} className="flex items-start gap-2">
              <span className="text-primary select-none">[{idx + 1}]</span>
              <span className="text-slate-200">{ev}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Actionable Script Console */}
      <div className="space-y-3">
        <h3 className="text-xs font-bold uppercase tracking-wider text-slate-400 flex items-center gap-1.5">
          <Terminal className="w-3.5 h-3.5 text-primary" />
          <span>Готовые команды исправления (PowerShell)</span>
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {fixRecipes.map((recipe, idx) => (
            <div
              key={idx}
              className="p-4 rounded-xl bg-surface border border-surface-border space-y-2.5 hover:border-primary/40 transition-all flex flex-col justify-between"
            >
              <div>
                <h4 className="text-xs font-bold text-white">{recipe.title}</h4>
                <p className="text-[11px] text-slate-400 mt-1 leading-snug">
                  {recipe.desc}
                </p>
              </div>

              <div className="flex items-center justify-between p-2 rounded-lg bg-slate-950 border border-surface-border font-mono text-[11px] text-primary">
                <span className="truncate mr-2">{recipe.cmd.split("\n")[0]}</span>
                <button
                  onClick={() => copyCommand(recipe.cmd, idx)}
                  className="p-1.5 rounded bg-surface-elevated hover:bg-surface-border text-slate-300 hover:text-white transition-colors shrink-0"
                  title="Скопировать команду в буфер"
                >
                  {copiedIndex === idx ? (
                    <Check className="w-3.5 h-3.5 text-emerald-400" />
                  ) : (
                    <Copy className="w-3.5 h-3.5" />
                  )}
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
