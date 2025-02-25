use crate::battery;
use crate::config;
use crate::power;
use crate::system;
use crate::windows;
use serde::Deserialize;
use serde::Serialize;
use tauri::AppHandle;
use tauri::Emitter;
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct EventChannel {
    battery: bool,
    system: bool,
    power: bool,
    config: bool,
    log: bool,
    history: bool,
}
impl EventChannel {
    pub fn new() -> Self {
        EventChannel::default()
    }

    pub async fn emit_service_update(handler: &AppHandle, current: &SessionState) {}
    pub async fn emit_ui_update(handler: &AppHandle, current: &SessionState) {
        if current.is_min_tray {
            return;
        }
        if current.channel.battery {
            let payload = &current.battery;
            handler.emit("battery_info_updated", payload).unwrap();
        }
        if current.channel.power {
            let payload = &current.power;
            handler.emit("power_info_updated", payload).unwrap();
        }
        if current.channel.system {
            let payload = &current.system;
            handler.emit("system_info_updated", payload).unwrap();
        }
    }
}
#[derive(Default)]
pub struct SessionState {
    pub is_admin: bool,
    pub is_min_tray: bool,
    pub config: config::Config,
    pub battery: battery::BatteryInfo,
    pub system: system::SystemInfo,
    pub power: power::PowerInfo,
    pub power_lock: power::PowerLock,
    pub channel: EventChannel,
}
impl SessionState {
    pub fn new(config: config::Config) -> Self {
        let mut system = system::SystemInfo::new();
        let is_admin = windows::is_admin();
        Self {
            is_admin,
            is_min_tray: false,
            config,
            battery: battery::BatteryInfo::new(),
            power_lock: power::PowerLock::new(),
            channel: EventChannel::new(),
            power: match system.support_power_set && is_admin {
                true => match power::PowerInfo::new() {
                    Ok(val) => val,
                    Err(_v) => {
                        system.support_power_set = false;
                        power::PowerInfo::default()
                    }
                },
                false => power::PowerInfo::default(),
            },
            system,
        }
    }
}
