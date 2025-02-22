use std::sync::Arc;

use crate::config;
use crate::power;
use crate::session;
use crate::system;
use crate::windows;
use tauri::Emitter;
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
    //let state = state.lock().await;
    Ok(windows::is_admin())
}
#[command]
pub async fn get_powerinfo(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
) -> Result<power::PowerInfo, ()> {
    let mut state = state.lock().await;
    Ok(
        match windows::is_admin() && state.system.support_power_set {
            true => match power::PowerInfo::new() {
                Ok(val) => {
                    state.system.support_power_set = true;
                    val
                }
                Err(_) => {
                    state.system.support_power_set = false;
                    power::PowerInfo::default()
                }
            },
            false => power::PowerInfo::default(),
        },
    )
}

#[command]
pub async fn exec_elevate_self(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
) -> Result<bool, ()> {
    windows::elevate_self();
    Ok(true)
}
#[command]
pub async fn set_power_limit(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
    limit: power::PowerLimit,
) -> Result<(bool, Option<power::PowerInfo>), ()> {
    let state = state.lock().await;
    Ok(
        match windows::is_admin() && state.system.support_power_set {
            true => match power::set_limit(&limit) {
                Ok(_) => {
                    let info = power::PowerInfo::new().unwrap();
                    app_handle.emit("power_info_updated", &info).unwrap();
                    if info.stapm_limit == limit.stapm_limit
                        && info.slow_limit == limit.slow_limit
                        && info.fast_limit == limit.fast_limit
                    {
                        (true, Some(info))
                    } else {
                        (false, Some(info))
                    }
                }
                Err(_) => (false, None),
            },
            false => (false, None),
        },
    )
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
) -> Result<(bool), ()> {
    let mut state = state.lock().await;
    if windows::is_admin() && state.system.support_power_set {
        state.power_lock.enable = lock;
        state.power_lock.limit = limit;
        Ok(true)
    } else {
        Ok(false)
    }
}
