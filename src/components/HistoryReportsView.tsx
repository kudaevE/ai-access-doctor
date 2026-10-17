import React from "react";
import {
  History,
  Download,
  Copy,
  Check,
  Shield,
  Trash2,
  FileText,
  Calendar,
  CheckCircle2,
  AlertTriangle,
  XCircle,
} from "lucide-react";
import { HistoryRecordSummary } from "../types";

interface HistoryReportsViewProps {
  history: HistoryRecordSummary[];
  onExportCurrent: (format: "markdown" | "json", sanitize: boolean) => Promise<string>;
  currentReportId?: string;
}

export const HistoryReportsView: React.FC<HistoryReportsViewProps> = ({
  history,
  onExportCurrent,
  currentReportId,
}) => {
  const [exportFormat, setExportFormat] = React.useState<"markdown" | "json">("markdown");
  const [sanitize, setSanitize] = React.useState(true);
  const [exportedText, setExportedText] = React.useState<string>("");
  const [copied, setCopied] = React.useState(false);
  const [loadingExport, setLoadingExport] = React.useState(false);

  const handleGenerateExport = async () => {
    setLoadingExport(true);
    try {
      const content = await onExportCurrent(exportFormat, sanitize);
      setExportedText(content);
    } catch (e) {
      console.error(e);
    } finally {
      setLoadingExport(false);
    }
  };

  const copyToClipboard = () => {
    navigator.clipboard.writeText(exportedText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const downloadFile = () => {
    const ext = exportFormat === "markdown" ? "md" : "json";
    const mime = exportFormat === "markdown" ? "text/markdown" : "application/json";
    const blob = new Blob([exportedText], { type: mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `ai-access-doctor-report-${Date.now()}.${ext}`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-bold text-white tracking-tight">
          Отчеты и история диагностик
        </h2>
        <p className="text-xs text-slate-400 mt-0.5">
          Все результаты сохраняются в локальную базу данных SQLite без отправки в облако.
        </p>
      </div>

      {/* Export Report Generator */}
      <div className="p-5 rounded-2xl bg-surface border border-surface-border space-y-4 shadow-sm">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between pb-3 border-b border-surface-border gap-2">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-primary/10 border border-primary/20 text-primary">
              <Download className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-sm font-bold text-white">Экспорт отчета текущей диагностики</h3>
              <p className="text-[11px] text-slate-400">Формирование документации для техподдержки или архива</p>
            </div>
          </div>

          {/* Format and Sanitize controls */}
          <div className="flex items-center gap-2">
            <div className="flex rounded-lg bg-surface-elevated border border-surface-border p-0.5 text-xs">
              <button
                onClick={() => setExportFormat("markdown")}
                className={`px-2.5 py-1 rounded-md font-medium transition-colors ${
                  exportFormat === "markdown"
                    ? "bg-primary/20 text-primary border border-primary/30"
                    : "text-slate-400 hover:text-white"
                }`}
              >
                Markdown (.md)
              </button>
              <button
                onClick={() => setExportFormat("json")}
                className={`px-2.5 py-1 rounded-md font-medium transition-colors ${
                  exportFormat === "json"
                    ? "bg-primary/20 text-primary border border-primary/30"
                    : "text-slate-400 hover:text-white"
                }`}
              >
                JSON (.json)
              </button>
            </div>

            <label className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-surface-elevated border border-surface-border text-xs text-slate-300 cursor-pointer select-none">
              <input
                type="checkbox"
                checked={sanitize}
                onChange={(e) => setSanitize(e.target.checked)}
                className="rounded border-slate-700 text-primary focus:ring-0"
              />
              <Shield className="w-3.5 h-3.5 text-emerald-400" />
              <span>Обезличить данные</span>
            </label>

            <button
              onClick={handleGenerateExport}
              disabled={loadingExport || !currentReportId}
              className="px-3.5 py-1.5 rounded-lg bg-primary hover:bg-primary-hover text-slate-950 font-bold text-xs transition-colors shadow-sm disabled:opacity-50"
            >
              {loadingExport ? "Сборка..." : "Сформировать"}
            </button>
          </div>
        </div>

        {/* Exported preview & download actions */}
        {exportedText && (
          <div className="space-y-3 pt-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-mono text-slate-400">Предпросмотр отчета:</span>
              <div className="flex items-center gap-2">
                <button
                  onClick={copyToClipboard}
                  className="flex items-center gap-1.5 px-3 py-1 rounded-md bg-surface-elevated hover:bg-surface-border text-xs text-slate-300 hover:text-white transition-colors"
                >
                  {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                  <span>{copied ? "Скопировано!" : "Копировать"}</span>
                </button>
                <button
                  onClick={downloadFile}
                  className="flex items-center gap-1.5 px-3 py-1 rounded-md bg-emerald-600 hover:bg-emerald-500 text-xs text-white font-medium transition-colors"
                >
                  <Download className="w-3.5 h-3.5" />
                  <span>Скачать файл</span>
                </button>
              </div>
            </div>

            <pre className="p-4 rounded-xl bg-slate-950 border border-surface-border text-slate-300 font-mono text-xs overflow-x-auto max-h-60">
              {exportedText}
            </pre>
          </div>
        )}
      </div>

      {/* History Table */}
      <div className="space-y-3">
        <h3 className="text-sm font-bold text-white flex items-center gap-2">
          <History className="w-4 h-4 text-primary" />
          <span>Локальный журнал диагностических запусков ({history.length})</span>
        </h3>

        {history.length === 0 ? (
          <div className="p-8 rounded-xl bg-surface border border-surface-border text-center text-xs text-slate-400">
            История запусков пока пуста. Запустите вашу первую диагностику!
          </div>
        ) : (
          <div className="overflow-x-auto rounded-xl border border-surface-border bg-slate-950/60">
            <table className="w-full text-left text-xs font-mono">
              <thead>
                <tr className="border-b border-surface-border text-[10px] uppercase tracking-wider text-slate-300 bg-surface-elevated/40">
                  <th className="py-2.5 px-3">Дата и время</th>
                  <th className="py-2.5 px-3">Статус</th>
                  <th className="py-2.5 px-3">Код проблемы</th>
                  <th className="py-2.5 px-3">Вердикт</th>
                  <th className="py-2.5 px-3">Результаты</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-surface-border/50 text-[11px]">
                {history.map((rec) => (
                  <tr key={rec.id} className="hover:bg-surface-elevated/20 transition-colors">
                    <td className="py-2.5 px-3 text-slate-300 flex items-center gap-1.5">
                      <Calendar className="w-3.5 h-3.5 text-slate-400" />
                      <span>{new Date(rec.timestamp).toLocaleString()}</span>
                    </td>
                    <td className="py-2.5 px-3">
                      <span
                        className={`px-1.5 py-0.5 rounded text-[10px] font-sans ${
                          rec.verdict_state === "HEALTHY"
                            ? "bg-emerald-500/20 text-emerald-300"
                            : rec.verdict_state === "WARNING"
                            ? "bg-amber-500/20 text-amber-300"
                            : "bg-rose-500/20 text-rose-300 font-bold"
                        }`}
                      >
                        {rec.verdict_state}
                      </span>
                    </td>
                    <td className="py-2.5 px-3 text-slate-200">{rec.primary_issue_code}</td>
                    <td className="py-2.5 px-3 font-sans text-slate-300 max-w-xs truncate" title={rec.title_ru}>
                      {rec.title_ru}
                    </td>
                    <td className="py-2.5 px-3 text-slate-300">
                      <span className="text-emerald-400">{rec.passed_count}✓</span> /{" "}
                      <span className="text-amber-400">{rec.warning_count}!</span> /{" "}
                      <span className="text-rose-400">{rec.failed_count}✗</span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
};
