import React, { useState, useEffect } from "react";
import {
  FileText,
  AlertTriangle,
  CheckCircle2,
  Trash2,
  FolderOpen,
  RefreshCw,
  ShieldCheck,
  ShieldAlert,
  Info,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { HostsDoctorReport } from "../types";

export const HostsDoctorSection: React.FC = () => {
  const [report, setReport] = useState<HostsDoctorReport | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [cleaning, setCleaning] = useState<boolean>(false);
  const [msg, setMsg] = useState<{ text: string; isError: boolean } | null>(null);

  const fetchReport = async () => {
    setLoading(true);
    try {
      const res = await invoke<HostsDoctorReport>("get_hosts_report");
      setReport(res);
    } catch (e: any) {
      console.error("Failed to get hosts report:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchReport();
  }, []);

  const handleClean = async () => {
    setCleaning(true);
    setMsg(null);
    try {
      const res = await invoke<string>("clean_ai_hosts");
      setMsg({ text: res, isError: false });
      await fetchReport();
    } catch (e: any) {
      setMsg({ text: `Ошибка очистки: ${e.toString()}`, isError: true });
    } finally {
      setCleaning(false);
    }
  };

  const handleOpenFolder = async () => {
    try {
      const res = await invoke<string>("open_etc_folder");
      setMsg({ text: res, isError: false });
    } catch (e: any) {
      setMsg({ text: `Ошибка: ${e.toString()}`, isError: true });
    }
  };

  return (
    <div className="bg-surface/50 border border-border/50 rounded-2xl p-6 space-y-5 backdrop-blur-xl">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="p-2.5 rounded-xl bg-sky-500/10 text-sky-400 border border-sky-500/20">
            <FileText className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="font-bold text-white text-base">
                Hosts File Doctor (Системный файл hosts)
              </h3>
              <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-sky-500/20 text-sky-300">
                C:\...\etc\hosts
              </span>
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Аудит скрытых статических IP-адресов, которые могут перебивать DNS и блокировать Antigravity или Gemini
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={fetchReport}
            disabled={loading}
            className="p-2 rounded-xl bg-surface-elevated hover:bg-slate-700/50 border border-border/50 text-slate-300 text-xs transition-all active:scale-95 disabled:opacity-50"
            title="Обновить отчет hosts"
          >
            <RefreshCw className={`w-4 h-4 ${loading ? "animate-spin" : ""}`} />
          </button>
        </div>
      </div>

      {msg && (
        <div
          className={`p-3.5 rounded-xl border text-xs flex items-center justify-between ${
            msg.isError
              ? "bg-rose-500/10 border-rose-500/30 text-rose-300"
              : "bg-emerald-500/10 border-emerald-500/30 text-emerald-300"
          }`}
        >
          <span>{msg.text}</span>
          <button onClick={() => setMsg(null)} className="underline opacity-75 hover:opacity-100 ml-4">
            Закрыть
          </button>
        </div>
      )}

      {/* Status Banner */}
      <div
        className={`p-4 rounded-xl border flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 ${
          report?.suspicious_entries_count && report.suspicious_entries_count > 0
            ? "bg-rose-950/20 border-rose-500/40"
            : "bg-emerald-950/20 border-emerald-500/30"
        }`}
      >
        <div className="flex items-center gap-3">
          {report?.suspicious_entries_count && report.suspicious_entries_count > 0 ? (
            <AlertTriangle className="w-5 h-5 text-rose-400 shrink-0" />
          ) : (
            <CheckCircle2 className="w-5 h-5 text-emerald-400 shrink-0" />
          )}
          <span className="text-xs text-slate-200 font-medium">
            {report?.status_ru || "Анализ файла hosts..."}
          </span>
        </div>

        <div className="flex items-center gap-2 shrink-0 w-full sm:w-auto">
          {report?.ai_entries_count && report.ai_entries_count > 0 ? (
            <button
              onClick={handleClean}
              disabled={cleaning}
              className="flex-1 sm:flex-none px-4 py-2 rounded-xl bg-rose-500/20 hover:bg-rose-500/30 border border-rose-500/40 text-rose-300 font-semibold text-xs flex items-center justify-center gap-2 active:scale-95 transition-all disabled:opacity-50"
            >
              <Trash2 className="w-3.5 h-3.5" />
              {cleaning ? "Очистка..." : "Очистить записи ИИ"}
            </button>
          ) : null}

          <button
            onClick={handleOpenFolder}
            className="flex-1 sm:flex-none px-3.5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-300 text-xs font-semibold flex items-center justify-center gap-2 active:scale-95 transition-all"
          >
            <FolderOpen className="w-3.5 h-3.5" />
            Открыть etc
          </button>
        </div>
      </div>

      {/* Entries Table if any */}
      {report?.entries && report.entries.length > 0 && (
        <div className="overflow-x-auto rounded-xl border border-border/40">
          <table className="w-full text-left text-xs">
            <thead className="bg-surface-elevated/60 text-slate-400 border-b border-border/40 font-mono">
              <tr>
                <th className="py-2.5 px-3 font-medium">Стр.</th>
                <th className="py-2.5 px-3 font-medium">IP-адрес</th>
                <th className="py-2.5 px-3 font-medium">Домен</th>
                <th className="py-2.5 px-3 font-medium">Примечание / Статус</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border/20 text-slate-300 font-mono">
              {report.entries.map((item, i) => (
                <tr
                  key={i}
                  className={`hover:bg-surface-elevated/30 ${
                    item.is_suspicious_or_dead ? "bg-rose-500/5" : ""
                  }`}
                >
                  <td className="py-2.5 px-3 text-slate-500">{item.line_number}</td>
                  <td
                    className={`py-2.5 px-3 font-bold ${
                      item.is_suspicious_or_dead ? "text-rose-400" : "text-emerald-400"
                    }`}
                  >
                    {item.ip}
                  </td>
                  <td className="py-2.5 px-3 text-sky-300">{item.domain}</td>
                  <td className="py-2.5 px-3 font-sans text-slate-400 text-[11px]">
                    {item.note_ru}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};
