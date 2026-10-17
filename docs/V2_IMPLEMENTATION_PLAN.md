# AI Access Doctor V2.0 — План реализации и дорожная карта

План структурирован по приоритетам в строгом соответствии с требованиями Master Prompt.

---

## Приоритет P0 — Фундамент надежности, дизайна и детектив ошибок

### Этап 0: Аудит и фиксация базовой версии
- [x] Провести аудит V1.0 и сформировать `docs/V2_AUDIT.md`.
- [x] Зафиксировать Git checkpoint (`master`).
- [x] Подготовить дорожную карту `docs/V2_IMPLEMENTATION_PLAN.md`.

### Этап 1: Обновление дизайн-системы и токенов (Linear / Raycast / Vercel Dark)
- [ ] Обновить `tailwind.config.js` и `src/index.css`:
  - Canvas background: `#0B1019`
  - Sidebar background: `#101722`
  - Surface cards: `#151E2D`
  - Elevated elements: `#1C2839`
  - Borders: `#223047`
  - Accent Primary: `#39BDF8` (Sky blue)
  - Success: `#22C997` (Emerald)
  - Warning: `#F4B544` (Amber)
  - Critical: `#F26778` (Rose)
  - Шрифт: `Inter`, `system-ui`, моноширинный `JetBrains Mono` / `Consolas`.
- [ ] Оформить премиальные компоненты: бейджи статусов, акцентные карточки, компактные переключатели, микроанимации.

### Этап 2: Редизайн Дашборда и прозрачность точности
- [ ] Заголовок: **Центр диагностики** / «Мониторинг подключения и состояния AI-инструментов».
- [ ] 5 ключевых карточек метрик в верхней строке:
  1. **Состояние подключений** (Бейдж статуса + пояснение).
  2. **Доступные AI-сервисы** (например, 5/5 доступно).
  3. **Активные предупреждения** (Количество предупреждений с быстрой ссылкой).
  4. **Средняя задержка** (Милисекунды с цветовым градиентом качества).
  5. **Последняя диагностика** (Время последнего прогона или индикатор ожидания).
- [ ] Информационный блок объективного вердикта:
  - Устранить необоснованные числовые проценты уверенности.
  - Ввести шкалу силы доказательств: `CONFIRMED_FACT` (Факт подтвержден прямым тестом), `STRONGLY_SUPPORTED` (Высокая вероятность), `HYPOTHESIS`, `INCONCLUSIVE`.
  - Эксплицитное информирование о ситуации, когда DNS провайдера подменяется, но нивелируется прокси-сервером.
- [ ] Расширенная сетка сервисов с 5 состояниями (`AVAILABLE`, `AUTH REQUIRED`, `DEGRADED`, `UNAVAILABLE`, `UNKNOWN`).
- [ ] Интерактивный таймлайн-график задержек (Latency Sparkline/Chart) на основе сохраненной истории.

### Этап 3: Детектив ошибок (Smart Error Detective)
- [ ] Разработка компонента `ErrorDetectiveView.tsx` и Rust-обработчика `analyze_error_log`:
  - Текстовое поле для вставки сообщения об ошибке из консоли / лога (Cursor, VS Code, Python, Node.js, Git).
  - Кнопка импорта файлов логов (`.log`, `.txt`, `.json`).
  - Локальный анализатор регулярных выражений:
    - HTTP-ошибки: `400 Bad Request`, `401 Unauthorized / Invalid API Key`, `403 Forbidden / Cloudflare Turnstile / Geo Block`, `404 Not Found`, `408 Request Timeout`, `429 Too Many Requests / Quota Exceeded`, `500 Internal Server Error`, `502 Bad Gateway`, `504 Gateway Timeout`.
    - Сокетные ошибки: `ECONNRESET`, `ETIMEDOUT`, `ENOTFOUND`, `ECONNREFUSED`, `EHOSTUNREACH`.
    - TLS/SSL ошибки: `UNABLE_TO_VERIFY_LEAF_SIGNATURE`, `CERT_HAS_EXPIRED`, `DEPTH_ZERO_SELF_SIGNED_CERT`, `SSL_ERROR_SYSCALL`.
    - Ошибки прокси: `Proxy Authentication Required (407)`, `SOCKS5 connect failed`, `WinINet connection refused`.
  - Санитайзер секретов в реальном времени (маскирование паролей, токенов, ключей).
  - Структурированный вывод:
    1. **Человеческое значение ошибки** (на русском языке, без жаргона).
    2. **Подтвержденные факты**.
    3. **Вероятная причина**.
    4. **Рекомендуемые шаги решения**.

---

## Приоритет P1 — Сравнительный анализ, таймлайн, расширенные отчеты и IDE Doctor

### Этап 4: Сравнительный анализ каналов (Connection Comparison)
- [ ] Модуль `ConnectionComparisonView.tsx` и бэкенд `test_connection_matrix`:
  - Параллельное тестирование сервисов по разным маршрутам:
    1. Прямое соединение (Direct Direct Egress)
    2. Системный прокси Windows (WinINet)
    3. Пользовательский прокси (HTTP/SOCKS5)
    4. DNS-over-HTTPS (DoH Cloudflare / Google)
    5. Виртуальный туннель (VPN/TUN)
  - Сравнительная таблица доступности и задержки по каждому каналу для мгновенного ответа на вопрос: «Почему работает в браузере, но не в терминале?».

### Этап 5: Таймлайн изменений системы (System Change Timeline)
- [ ] Модуль `ChangeTimelineView.tsx` и вычисление диффов между прогонами:
  - Отслеживание изменений сетевых шлюзов, публичного IP, статуса прокси, DNS-серверов и переменных окружения во времени.
  - Фиксация момента появления сбоя относительно предыдущего успешного запуска.

### Этап 6: Расширенные отчеты и экспорт (Advanced Reports)
- [ ] Двухуровневый экспорт:
  - **Executive Summary** (краткая выжимка для неспециалистов).
  - **Technical Markdown / JSON / HAR-like log** (полные дампы заголовков, сокетов, маршрутов с очисткой токенов).

### Этап 7: IDE Doctor
- [ ] Модуль `IdeDoctorView.tsx` и бэкенд `diagnose_ide_configs`:
  - Аудит параметров подключения и прокси для:
    - Cursor (`settings.json`, `http.proxy`, `http.proxyStrictSSL`)
    - Visual Studio Code (`settings.json`, расширения)
    - Windsurf (`settings.json`)
    - Zed (`settings.json`)
    - JetBrains (AI Assistant, `options/other.xml`, `proxy.xml`)
    - Antigravity CLI / Sidecar

---

## Приоритет P2 — Безопасное устранение проблем, MCP Doctor и мониторинг

### Этап 8: Безопасный движок устранения проблем (Safe Fix Engine)
- [ ] Модуль `SafeFixEngine` в Rust и UI модальное окно предварительного просмотра:
  - Строгий список безопасных разрешенных действий (Allowlist):
    - `FLUSH_DNS` (`ipconfig /flushdns`)
    - `ALIGN_ENV_PROXY` (Синхронизация переменной окружения `HTTP_PROXY` с рабочим портом WinINet)
    - `CLEAR_STALE_ENV_PROXY` (Удаление неработающей переменной `HTTP_PROXY=53200`)
    - `RESET_WINSOCK_CATALOG` (Подготовка команды сброса)
  - Предварительный просмотр (Dry-Run): показ точной команды перед выполнением.
  - Обязательное подтверждение пользователя с фиксацией контрольной точки отката (Rollback).

### Этап 9: MCP Doctor (Model Context Protocol)
- [ ] Модуль `McpDoctorView.tsx`:
  - Инспекция путей конфигурационных файлов:
    - `claude_desktop_config.json`
    - `cursor/mcp.json`
    - `antigravity/mcp_servers.json`
  - Валидация JSON, проверка путей к исполняемым файлам (node, python, npx), тестовый зонд SSE и stdio.

### Этап 10: Фоновый мониторинг (Background Heartbeat)
- [ ] Опциональный фоновый таймер проверки доступности (с интервалом 5/15/30 мин) с минимизацией потребления ресурсов CPU/RAM.

---

## Приоритет P3 — Оповещения, коммерческая архитектура и релиз

### Этап 11: Telegram-оповещения
- [ ] Настройка бота Telegram (токен бота + chat_id) для отправки алертов при критическом отказе каналов.

### Этап 12: Архитектура редакций (Editions Architecture)
- [ ] Переключатель функционала в настройках: Community, Professional, Team, Enterprise (демонстрация модульности и политик соответствия).

### Этап 13: Полное тестирование, верификация и сборка установщика
- [ ] Запуск `cargo test` (проверка всех тестов).
- [ ] Запуск `npm run build` (проверка типов TypeScript и сборки Vite).
- [ ] Сборка релизного инсталлятора Windows `tauri build`.
- [ ] Финальный отчет и документация.
