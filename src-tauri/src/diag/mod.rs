pub mod antigravity_unlock;
pub mod proxy_bridge;
pub mod cli_proxy;
pub mod comparison;
pub mod dns;
pub mod flow_unlocker;
pub mod free_hub;
pub mod hosts_doctor;
pub mod ide;
pub mod mcp;
pub mod network;
pub mod proxy;
pub mod speed_benchmark;
pub mod tls;
pub mod vpn;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn silent_command(program: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}
