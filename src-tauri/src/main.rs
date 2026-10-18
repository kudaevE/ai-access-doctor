#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let log_msg = format!("PANIC at {:?}: {}\n", chrono::Utc::now(), info);
        let log_dir = std::env::var("LOCALAPPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir())
            .join("AI Access Doctor");
        let _ = std::fs::create_dir_all(&log_dir);
        let _ = std::fs::write(log_dir.join("startup_panic.log"), &log_msg);
        eprintln!("{}", log_msg);
    }));

    ai_access_doctor_lib::run();
}
