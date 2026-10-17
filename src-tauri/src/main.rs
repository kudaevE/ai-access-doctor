// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let log_msg = format!("PANIC at {:?}: {}\n", chrono::Utc::now(), info);
        let _ = std::fs::write("C:\\AI Access Doctor\\startup_panic.log", &log_msg);
        eprintln!("{}", log_msg);
    }));

    ai_access_doctor_lib::run();
}
