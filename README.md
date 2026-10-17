# AI Access Doctor (Windows 10/11 x64)

> **AI Access Doctor** — локальное Windows-приложение для доказательной диагностики сбоев сетевого подключения, авторизации и маршрутизации инструментов разработчика искусственного интеллекта (Google Antigravity, Gemini, OpenAI / Codex, Anthropic Claude, GitHub Developer Services).

---

## 1. Возможности и архитектура

AI Access Doctor заменяет типовые абстрактные ошибки (такие как `ETIMEDOUT`, `403 Forbidden`, `unable to get local issuer certificate`) на фактологический анализ корневых причин на русском и английском языках с оценкой достоверности и готовыми рецептами устранения.

### Диагностические модули:
- **Сеть и Стек (Network & Dual-Stack):** Аудит физических и виртуальных сетевых адаптеров, MTU, шлюзов, IPv4 и IPv6 связности с выявлением IPv6-blackhole.
- **DNS-Резолверы (DNS Comparison):** Одновременное сопоставление системного DNS с независимыми Do53/DoH резолверами (Cloudflare `1.1.1.1`, Google `8.8.8.8`, Quad9 `9.9.9.9`) для обнаружения цензуры, спуфинга и манипуляций NXDOMAIN.
- **Инспекция TLS и Сертификатов:** Прямой handshake TLS 1.2 / TLS 1.3 с проверкой доверия по WebPKI и выявлением антивирусных/корпоративных MITM-инспекторов (Kaspersky, ESET, Bitdefender, Zscaler, Fortinet).
- **Аудит Прокси (WinINet & WinHTTP):** Поиск скрытых конфликтов между системным прокси Windows (реестр `HKCU`) и переменными окружения (`HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`). Проверка доступности портов и детекция `502 Bad Gateway`.
- **VPN и Маршрутизация:** Выявление виртуальных адаптеров (WireGuard, Wintun, Tailscale, Cloudflare WARP, OpenVPN) и конфликтов метрик маршрута по умолчанию (`0.0.0.0/0`).
- **Профили ИИ-сервисов:** Выверенные эндпоинты для Google Antigravity, Gemini, OpenAI, Claude и GitHub. Четкое разграничение сетевого сбоя и штатной границы авторизации (HTTP 401/403 Auth Boundary).
- **Экспертный Rule Engine:** Детерминированные правила выявления корневых причин с расчетом процента достоверности (Confidence Level) и рекомендациями.
- **Локальное хранилище (SQLite):** Хранение истории диагностических запусков на локальном диске пользователя без отправки телеметрии в облако.
- **Обезличенный Экспорт:** Экспорт отчетов в Markdown и JSON с авто-санитайзером приватных ключей, токенов и имен учетных записей.

---

## 2. Системные требования

- **ОС:** Windows 10 или Windows 11 (x64)
- **Компоненты:** Microsoft Edge WebView2 Runtime (предустановлен в современных сборках Windows)
- **Для сборки из исходников:**
  - Rust 1.85+ (MSVC toolchain: `x86_64-pc-windows-msvc`)
  - Node.js LTS (v20+ / v22+ / v24+) и npm
  - Visual Studio 2022 C++ Build Tools & Windows SDK

---

## 3. Сборка и запуск для разработчиков

```powershell
# 1. Клонирование репозитория
git clone https://github.com/your-org/ai-access-doctor.git
cd "ai-access-doctor"

# 2. Установка зависимостей frontend
npm install

# 3. Запуск в режиме разработки с live-reload
npm run tauri dev

# 4. Запуск автоматических тестов ядра
cd src-tauri
cargo test
cd ..

# 5. Сборка релизного инсталлятора Windows
npm run tauri build
```

Готовый установщик и бинарный файл помещаются в директорию:
`src-tauri/target/release/bundle/nsis/` или `src-tauri/target/release/ai-access-doctor.exe`.

---

## 4. Лицензионное соответствие

При разработке анализировались архитектурные паттерны reference-репозиториев:
- `netutils-cli` (MIT License) — методы TLS-пробинга и реестра WinINet.
- `NetPulse` (MIT License) — асинхронные команды Tauri 2 и SQLite.
- `network-doctor` (Apache 2.0) — типизированные структуры проверок.
- `netok` (GPL v3.0) — **код не копировался**, заимствованы исключительно концепции локальной приватности и отсутствия телеметрии.
