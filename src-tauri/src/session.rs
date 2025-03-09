use crate::config;
use crate::windows;
use chrono::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use status::{Last, Status};
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

    pub async fn emit_service_update(handler: &AppHandle, current: &SessionState) {
        if let Some(v) = &current.battery {
            if v.state_changed {
                handler.emit("battery_state_changed", v).unwrap();
            }
        }
    }
    pub async fn emit_ui_update(handler: &AppHandle, current: &SessionState) {
        if current.is_min_tray {
            return;
        }
        if current.channel.battery {
            if let Some(v) = &current.battery {
                handler.emit("battery_info_updated", v).unwrap();
            }
        }
        if current.channel.power && current.power.is_some() {
            let payload = &current.power;
            handler.emit("power_info_updated", payload).unwrap();
        }
        if current.channel.system {
            let payload = &current.system;
            handler.emit("system_info_updated", payload).unwrap();
        }
    }
}

pub struct PowerLock {
    pub limit: power::PowerLimit,
    pub enable: bool,
    pub lastcheck: i64,
}
impl Default for PowerLock {
    fn default() -> Self {
        Self {
            limit: Default::default(),
            enable: false,
            lastcheck: Utc::now().timestamp(),
        }
    }
}
impl PowerLock {
    pub fn new() -> Self {
        PowerLock::default()
    }
}
#[derive(Default)]
pub struct SessionState {
    pub is_admin: bool,
    pub is_min_tray: bool,
    pub config: config::Config,
    pub battery: Option<battery::Status>,
    pub system: system::Status,
    pub power: Option<power::Status>,
    pub power_lock: PowerLock,
    pub channel: EventChannel,
}
impl SessionState {
    pub fn new(config: config::Config) -> Self {
        let mut system = system::Status::build().unwrap();
        let mut battery: Option<battery::Status> = None;
        if let Some(rows) = battery::Status::build() {
            if rows.len() > 0 {
                battery = Some(rows[0].clone());
            }
        }
        let is_admin = windows::is_admin();
        Self {
            is_admin,
            is_min_tray: false,
            config,
            battery,
            power_lock: PowerLock::new(),
            channel: EventChannel::new(),
            power: match system.support_power_set && is_admin {
                true => match power::Status::build() {
                    Some(val) => Some(val),
                    None => {
                        system.support_power_set = false;
                        None
                    }
                },
                false => None,
            },
            system,
        }
    }
}
