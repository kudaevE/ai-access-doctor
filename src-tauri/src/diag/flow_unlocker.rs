use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub const EXTENSION_MANIFEST: &str = r#"{
  "manifest_version": 3,
  "name": "Google Flow Unlocker (AI Access Doctor)",
  "version": "1.0.0",
  "description": "Автоматическая разблокировка Google Flow (Veo, Imagen 3) для Chrome и Edge",
  "content_scripts": [
    {
      "matches": ["https://flow.google.com/*"],
      "js": ["content.js"],
      "run_at": "document_start",
      "world": "MAIN",
      "all_frames": true
    }
  ]
}
"#;

pub const FLOW_UNLOCKER_SCRIPT: &str = r#"
(() => {
  'use strict';
  try {
    const hostname = (typeof window !== 'undefined' && window.location && window.location.hostname) || '';
    if (!hostname.includes('flow.google.com')) {
      return;
    }

    const w = typeof window !== 'undefined' ? window : (typeof unsafeWindow !== 'undefined' ? unsafeWindow : this);
    const RPCS = ['cPZSdc', 'KV2T2d', 'rThb8d', 'cO7JOb', 'md9xJf'];

    function patchRpcJson(text, id) {
      try {
        const value = JSON.parse(text);
        if (!Array.isArray(value)) return text;
        if (id === 'cPZSdc') {
          while (value.length < 32) value.push(null);
          value[30] = 1;
          value[31] = 1;
        } else if (RPCS.includes(id)) {
          if (value.length) value[0] = 1;
          else value.push(1);
        }
        return JSON.stringify(value);
      } catch { return text; }
    }

    function patchStream(raw) {
      if (typeof raw !== 'string' || !RPCS.some(id => raw.includes(id))) return raw;
      let start = raw.indexOf(")]}'\n\n");
      let prefix = '';
      if (start >= 0) { start += 6; prefix = raw.slice(0, start); }
      else {
        start = raw.indexOf('\n\n');
        if (start >= 0) { start += 2; prefix = raw.slice(0, start); }
        else start = 0;
      }
      const body = raw.slice(start);
      try {
        const data = JSON.parse(body);
        if (Array.isArray(data)) {
          let changed = false;
          for (const item of data) {
            if (Array.isArray(item) && RPCS.includes(item[1]) && typeof item[2] === 'string') {
              item[2] = patchRpcJson(item[2], item[1]); changed = true;
            }
          }
          if (changed) return prefix + JSON.stringify(data);
        }
      } catch {}

      let output = prefix, pos = start, changed = false;
      let iterations = 0;
      while (pos < raw.length && iterations < 50000) {
        iterations++;
        const nl = raw.indexOf('\n', pos);
        if (nl < 0) { output += raw.slice(pos); break; }
        const sizeText = raw.slice(pos, nl), size = Number(sizeText);
        if (!Number.isFinite(size) || size < 1 || nl + size > raw.length) { output += raw.slice(pos); break; }
        const chunkWithNl = raw.slice(nl, nl + size);
        try {
          const chunk = JSON.parse(chunkWithNl.slice(1));
          let chunkChanged = false;
          for (const item of chunk) {
            if (Array.isArray(item) && RPCS.includes(item[1]) && typeof item[2] === 'string') {
              item[2] = patchRpcJson(item[2], item[1]); chunkChanged = changed = true;
            }
          }
          output += chunkChanged ? `${1 + JSON.stringify(chunk).length}\n${JSON.stringify(chunk)}` : sizeText + chunkWithNl;
        } catch { output += sizeText + chunkWithNl; }
        pos = nl + size;
      }
      return changed ? output : raw;
    }

    function patchJson(data) {
      if (!data || typeof data !== 'object') return data;
      if (Array.isArray(data)) return data.map(patchJson);
      for (const key of Object.keys(data)) {
        if (key === 'availabilityState') data[key] = 'AVAILABLE';
        else if (['isCountrySupported', 'is_country_supported', 'isAgeSupported', 'is_age_supported'].includes(key)) data[key] = true;
        else if (data[key] && typeof data[key] === 'object') data[key] = patchJson(data[key]);
      }
      return data;
    }

    try {
      let wiz = w.WIZ_global_data || {};
      const forceFlag = target => {
        target.awbSEf = true;
        return new Proxy(target, { set(obj, key, value) { obj[key] = value; obj.awbSEf = true; return true; } });
      };
      wiz = forceFlag(wiz);
      Object.defineProperty(w, 'WIZ_global_data', {
        configurable: true, enumerable: true, get: () => wiz,
        set(value) { if (value && typeof value === 'object') wiz = forceFlag(value); }
      });
    } catch (e) {
      console.warn('[Flow Unlocker] WIZ_global_data patch warning:', e);
    }

    try {
      const proto = w.XMLHttpRequest?.prototype;
      if (proto) {
        for (const key of ['responseText', 'response']) {
          const desc = Object.getOwnPropertyDescriptor(proto, key);
          if (!desc?.get) continue;
          Object.defineProperty(proto, key, {
            configurable: true,
            get() {
              const value = desc.get.call(this);
              if (this.responseType && this.responseType !== 'text' && !(key === 'response' && this.responseType === '')) return value;
              return typeof value === 'string' ? patchStream(value) : value;
            }
          });
        }
      }
    } catch (e) {
      console.warn('[Flow Unlocker] XHR patch warning:', e);
    }

    try {
      const originalFetch = w.fetch;
      if (originalFetch) {
        w.fetch = async function(...args) {
          const request = args[0];
          const url = typeof request === 'string' ? request : request?.url || '';
          const response = await originalFetch.apply(this, args);
          if (url.includes('batchexecute')) {
            try {
              const headers = new Headers(response.headers); headers.delete('content-length');
              return new Response(patchStream(await response.text()), { status: response.status, statusText: response.statusText, headers });
            } catch { return response; }
          }
          if (['fetchToolAvailability', 'checkAppAvailability', 'trpc/general.'].some(part => url.includes(part))) {
            try {
              const text = await response.text(), headers = new Headers(response.headers); headers.delete('content-length');
              try {
                return new Response(JSON.stringify(patchJson(JSON.parse(text))), { status: 200, headers });
              } catch { return new Response(text, { status: response.status, statusText: response.statusText, headers }); }
            } catch { return response; }
          }
          return response;
        };
      }
    } catch (e) {
      console.warn('[Flow Unlocker] fetch patch warning:', e);
    }
    console.log('[AI Access Doctor] Google Flow Client-Side Unlocker active on', hostname);
  } catch (globalErr) {
    console.error('[AI Access Doctor] Flow Unlocker fatal error:', globalErr);
  }
})();
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowUnlockInfo {
    pub flow_url: String,
    pub script_code: String,
    pub bookmarklet: String,
    pub is_window_open: bool,
    pub extension_folder_path: String,
    pub default_browser_name: String,
}

fn get_browser_exe() -> Option<(String, &'static str)> {
    let chrome_paths = [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    ];
    for p in &chrome_paths {
        if std::path::Path::new(p).exists() {
            return Some((p.to_string(), "Google Chrome"));
        }
    }

    let edge_paths = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    ];
    for p in &edge_paths {
        if std::path::Path::new(p).exists() {
            return Some((p.to_string(), "Microsoft Edge"));
        }
    }

    None
}

pub fn get_flow_info(app: &AppHandle) -> FlowUnlockInfo {
    let is_window_open = app.get_webview_window("google-flow").is_some();

    let minified = FLOW_UNLOCKER_SCRIPT
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.starts_with("//"))
        .collect::<Vec<&str>>()
        .join(" ");

    let bookmarklet = format!("javascript:{}", minified);
    let browser_name = get_browser_exe()
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| "Основной браузер".to_string());

    FlowUnlockInfo {
        flow_url: "https://flow.google.com/".to_string(),
        script_code: FLOW_UNLOCKER_SCRIPT.trim().to_string(),
        bookmarklet,
        is_window_open,
        extension_folder_path: r"C:\AI Access Doctor\flow_unlocker_extension".to_string(),
        default_browser_name: browser_name,
    }
}

pub fn ensure_extension_files_exist() {
    let ext_dir = std::path::PathBuf::from(r"C:\AI Access Doctor\flow_unlocker_extension");
    let _ = std::fs::create_dir_all(&ext_dir);
    let manifest_path = ext_dir.join("manifest.json");
    if !manifest_path.exists() {
        let _ = std::fs::write(&manifest_path, EXTENSION_MANIFEST);
    }
    let content_path = ext_dir.join("content.js");
    let _ = std::fs::write(&content_path, FLOW_UNLOCKER_SCRIPT.trim());

    let user_js_path = std::path::PathBuf::from(r"C:\AI Access Doctor\flow-unlocker.user.js");
    if !user_js_path.exists() {
        let user_script = format!(
            "// ==UserScript==\n// @name Google Flow Unlocker\n// @match https://flow.google.com/*\n// @run-at document-start\n// @grant unsafeWindow\n// ==/UserScript==\n{}",
            FLOW_UNLOCKER_SCRIPT.trim()
        );
        let _ = std::fs::write(&user_js_path, user_script);
    }
}

pub fn launch_browser_with_flow() -> Result<String, String> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    use std::os::windows::process::CommandExt;

    ensure_extension_files_exist();

    let ext_dir = r"C:\AI Access Doctor\flow_unlocker_extension";
    let ext_arg = format!("--load-extension={}", ext_dir);

    if let Some((exe_path, browser_name)) = get_browser_exe() {
        Command::new(&exe_path)
            .arg(&ext_arg)
            .arg("https://flow.google.com/")
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Не удалось запустить {}: {}", browser_name, e))?;

        Ok(format!(
            "Google Flow открыт в {} с активированным модулем разблокировки!",
            browser_name
        ))
    } else {
        Command::new("cmd")
            .args(["/C", "start", "", "https://flow.google.com/"])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Не удалось открыть браузер по умолчанию: {}", e))?;

        Ok("Google Flow открыт в вашем браузере по умолчанию.".to_string())
    }
}

pub fn open_extension_folder() -> Result<String, String> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    use std::os::windows::process::CommandExt;

    ensure_extension_files_exist();

    Command::new("explorer")
        .arg(r"C:\AI Access Doctor\flow_unlocker_extension")
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("Не удалось открыть папку: {}", e))?;

    Ok("Папка с расширением открыта в Проводнике Windows.".to_string())
}

pub fn open_extensions_manager_in_browser() -> Result<String, String> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    use std::os::windows::process::CommandExt;

    if let Some((exe_path, _)) = get_browser_exe() {
        let manager_url = if exe_path.to_lowercase().contains("chrome") {
            "chrome://extensions"
        } else {
            "edge://extensions"
        };

        Command::new(&exe_path)
            .arg(manager_url)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Ошибка открытия менеджера расширений: {}", e))?;

        Ok("В браузере открыта страница управления расширениями (включите 'Режим разработчика' и перетащите папку).".to_string())
    } else {
        Err("Браузер не найден".to_string())
    }
}

pub fn launch_google_flow_window(app: &AppHandle) -> Result<String, String> {
    if let Some(win) = app.get_webview_window("google-flow") {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok("Окно Google Flow уже открыто и выведено на передний план.".to_string());
    }

    let url: tauri::Url = "https://flow.google.com/"
        .parse()
        .map_err(|e| format!("Неверный URL: {}", e))?;

    let builder = WebviewWindowBuilder::new(app, "google-flow", WebviewUrl::External(url))
        .title("Google Flow Studio — Разблокировано (AI Access Doctor)")
        .inner_size(1400.0, 900.0)
        .center()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .devtools(true)
        .initialization_script(FLOW_UNLOCKER_SCRIPT);

    let win = builder
        .build()
        .map_err(|e| format!("Ошибка создания окна Google Flow: {}", e))?;

    let _ = win.show();
    let _ = win.set_focus();

    Ok("Окно Google Flow успешно открыто со встроенным патчем региональной доступности!".to_string())
}
