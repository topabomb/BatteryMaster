use std::sync::Arc;

use crate::config;
use crate::power;
use crate::session;
use crate::system;
use crate::windows;
use log::{log, Level};
use tauri::{command, State};
use tokio::sync::Mutex;
#[command]
pub async fn get_config(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
) -> Result<config::Config, ()> {
    let state = state.lock().await;
    Ok(state.config)
}

#[command]
pub async fn set_config(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
    config: config::Config,
) -> Result<bool, ()> {
    let mut state = state.lock().await;
    state.config = config;
    config::save_config(&state.config).expect("set_config err.");
    config::set_autostart(&app_handle, config.auto_start);

    Ok(true)
}
#[command]
pub async fn get_isadmin(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
) -> Result<bool, ()> {
    let state = state.lock().await;
    Ok(state.is_admin)
}
#[command]
pub async fn get_powerinfo(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
) -> Result<power::PowerInfo, ()> {
    let mut state = state.lock().await;
    Ok(match state.is_admin && state.system.support_power_set {
        true => match power::PowerInfo::new() {
            Ok(val) => {
                state.system.support_power_set = true;
                val
            }
            Err(err) => {
                log!(Level::Warn, "command get_powerinfo err:{:?}", err);
                state.system.support_power_set = false;
                power::PowerInfo::default()
            }
        },
        false => power::PowerInfo::default(),
    })
}

#[command]
pub async fn exec_elevate_self(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
) -> Result<bool, ()> {
    let state = state.lock().await;
    if !state.is_admin {
        windows::elevate_self();
    }

    Ok(true)
}
#[command]
pub async fn set_power_limit(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
    limit: power::PowerLimit,
) -> Result<(bool, Option<power::PowerInfo>), ()> {
    let mut state = state.lock().await;
    if !state.is_admin || !state.system.support_power_set {
        return Ok((false, None));
    }
    let result = match power::set_limit(&limit) {
        Ok(_) => {
            state.power = power::PowerInfo::new().unwrap();
            if state.power.stapm_limit == limit.stapm_limit
                && state.power.slow_limit == limit.slow_limit
                && state.power.fast_limit == limit.fast_limit
            {
                (true, Some(state.power.clone()))
            } else {
                (false, Some(state.power.clone()))
            }
        }
        Err(_) => (false, None),
    };
    if result.0 {
        session::EventChannel::emit_ui_update(&app_handle, &state).await;
    }
    Ok(result)
}
#[command]
pub async fn get_system(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
) -> Result<system::SystemInfo, ()> {
    let state = state.lock().await;
    Ok(state.system.clone())
}
#[command]
pub async fn set_power_limit_lock(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
    lock: bool,
    limit: power::PowerLimit,
) -> Result<bool, ()> {
    let mut state = state.lock().await;
    if state.is_admin && state.system.support_power_set {
        state.power_lock.enable = lock;
        state.power_lock.limit = limit;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn set_event_channel(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
    setting: session::EventChannel,
) -> Result<bool, ()> {
    let mut state = state.lock().await;
    state.channel = setting;
    Ok(true)
}
