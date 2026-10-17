import React from "react";
import {
  Settings,
  ShieldCheck,
  Server,
  Info,
  Lock,
  Cpu,
  RefreshCw,
} from "lucide-react";

interface SettingsViewProps {
  proxyOverride: string;
  setProxyOverride: (val: string) => void;
  onClearHistory: () => void;
}

export const SettingsView: React.FC<SettingsViewProps> = ({
  proxyOverride,
  setProxyOverride,
  onClearHistory,
}) => {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-bold text-white tracking-tight">
          Параметры и гарантии безопасности
        </h2>
        <p className="text-xs text-slate-400 mt-0.5">
          Конфигурация тестовых прокси, управление хранилищем и аудит окружения Windows.
        </p>
      </div>

      {/* Custom Proxy Override Setting */}
      <div className="p-5 rounded-2xl bg-surface border border-surface-border space-y-4 shadow-sm">
        <div className="flex items-center gap-2.5 pb-3 border-b border-surface-border">
          <div className="p-2 rounded-xl bg-primary/10 border border-primary/20 text-primary">
            <Server className="w-5 h-5" />
          </div>
          <div>
            <h3 className="text-sm font-bold text-white">Переопределение тестового прокси (Proxy Override)</h3>
            <p className="text-[11px] text-slate-400">
              Позволяет принудительно протестировать подключение через указанный порт без изменения настроек Windows
            </p>
          </div>
        </div>

        <div className="space-y-2">
          <label className="text-xs text-slate-300 font-medium block">
            Адрес и порт прокси (HTTP / SOCKS5):
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={proxyOverride}
              onChange={(e) => setProxyOverride(e.target.value)}
              placeholder="Оставьте пустым для автоопределения или укажите http://127.0.0.1:10809"
              className="flex-1 px-3.5 py-2 rounded-xl bg-slate-950 border border-surface-border text-xs text-slate-200 font-mono focus:outline-none focus:border-primary"
            />
            {proxyOverride && (
              <button
                onClick={() => setProxyOverride("")}
                className="px-3 py-2 rounded-xl bg-surface-elevated hover:bg-surface-border text-xs text-slate-400 hover:text-white transition-colors"
              >
                Сбросить
              </button>
            )}
          </div>
          <p className="text-[11px] text-slate-400 leading-relaxed">
            Если поле заполнено, запросы к эндпоинтам ИИ будут отправляться через этот прокси. Если пусто — используется автодетект системного прокси Windows или переменных окружения.
          </p>
        </div>
      </div>

      {/* Safety & Privacy Policy Card */}
      <div className="p-5 rounded-2xl bg-surface border border-surface-border space-y-3">
        <div className="flex items-center gap-2.5 pb-2 border-b border-surface-border">
          <ShieldCheck className="w-5 h-5 text-emerald-400" />
          <h3 className="text-sm font-bold text-white">Гарантии конфиденциальности и безопасности</h3>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 pt-1 text-xs text-slate-300">
          <div className="flex items-start gap-2.5 p-3 rounded-xl bg-slate-950/40 border border-surface-border">
            <Lock className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
            <div>
              <span className="font-bold text-white block">100% Local-First (Без облака)</span>
              <span className="text-[11px] text-slate-400">
                Никакие результаты проверок, IP-адреса или системные данные не передаются на внешние серверы телеметрии.
              </span>
            </div>
          </div>

          <div className="flex items-start gap-2.5 p-3 rounded-xl bg-slate-950/40 border border-surface-border">
            <ShieldCheck className="w-4 h-4 text-primary shrink-0 mt-0.5" />
            <div>
              <span className="font-bold text-white block">Безопасный режим Read-Only</span>
              <span className="text-[11px] text-slate-400">
                Приложение выполняет только аудит. Системный реестр, DNS, адаптеры и брандмауэр не изменяются в автоматическом режиме.
              </span>
            </div>
          </div>

          <div className="flex items-start gap-2.5 p-3 rounded-xl bg-slate-950/40 border border-surface-border">
            <Info className="w-4 h-4 text-accent-cyan shrink-0 mt-0.5" />
            <div>
              <span className="font-bold text-white block">Никакого сбора секретов</span>
              <span className="text-[11px] text-slate-400">
                Не сканируются токены, cookies браузеров или пароли. Диагностика проверяет исключительно доступность сетевого протокола.
              </span>
            </div>
          </div>

          <div className="flex items-start gap-2.5 p-3 rounded-xl bg-slate-950/40 border border-surface-border">
            <Cpu className="w-4 h-4 text-accent-purple shrink-0 mt-0.5" />
            <div>
              <span className="font-bold text-white block">Обезличивание при экспорте</span>
              <span className="text-[11px] text-slate-400">
                Функция санитайзера автоматически удаляет имена учетных записей Windows и ключи из экспортируемых отчетов.
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* System Audit Information */}
      <div className="p-5 rounded-2xl bg-surface border border-surface-border space-y-3">
        <h3 className="text-sm font-bold text-white flex items-center gap-2">
          <Info className="w-4 h-4 text-primary" />
          <span>Сведения о среде выполнения</span>
        </h3>

        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-xs font-mono">
          <div className="p-3 rounded-xl bg-slate-950 border border-surface-border">
            <span className="text-[10px] text-slate-400 uppercase block font-sans">Операционная система</span>
            <span className="text-slate-200 font-bold">Windows 11 x64</span>
          </div>
          <div className="p-3 rounded-xl bg-slate-950 border border-surface-border">
            <span className="text-[10px] text-slate-400 uppercase block font-sans">WebView2 Runtime</span>
            <span className="text-slate-200 font-bold">155.0.4283.60</span>
          </div>
          <div className="p-3 rounded-xl bg-slate-950 border border-surface-border">
            <span className="text-[10px] text-slate-400 uppercase block font-sans">Ядро диагностики</span>
            <span className="text-primary font-bold">Rust / Tokio (Tauri 2)</span>
          </div>
          <div className="p-3 rounded-xl bg-slate-950 border border-surface-border">
            <span className="text-[10px] text-slate-400 uppercase block font-sans">Локальная БД</span>
            <span className="text-emerald-400 font-bold">SQLite 3 (rusqlite)</span>
          </div>
        </div>

        <div className="pt-2 flex justify-end">
          <button
            onClick={onClearHistory}
            className="px-3.5 py-1.5 rounded-lg border border-rose-500/30 bg-rose-500/10 hover:bg-rose-500/20 text-rose-300 text-xs font-medium transition-colors"
          >
            Очистить локальный журнал проверок
          </button>
        </div>
      </div>
    </div>
  );
};
