use std::sync::Arc;

use crate::config;
use crate::session;
use crate::windows;
use chrono::Duration;
use chrono::Utc;
use log::{log, Level};
use status::{Last, Status};
use system::*;
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
) -> Result<power::Status, String> {
    let mut state = state.lock().await;
    if state.is_admin && state.system.support_power_set && state.power.is_some() {
        let info = state.power.as_mut().unwrap();
        info.last();
        Ok(info.clone())
    } else {
        log!(Level::Warn, "command get_powerinfo err.");
        state.system.support_power_set = false;
        Err("get_powerinfo err.".to_string())
    }
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
) -> Result<(bool, Option<power::Status>), ()> {
    let mut state = state.lock().await;
    if !state.is_admin || !state.system.support_power_set || state.power.is_none() {
        return Ok((false, None));
    }
    let result = match power::set_limit(&limit) {
        Ok(_) => {
            let info = state.power.as_mut().unwrap();
            info.last();
            if info.stapm_limit == limit.stapm_limit
                && info.slow_limit == limit.slow_limit
                && info.fast_limit == limit.fast_limit
            {
                (true, Some(info.clone()))
            } else {
                (false, Some(info.clone()))
            }
        }
        Err(_) => (false, None),
    };
    if result.0 {
        session::EventChannel::emit_ui_update(&app_handle, &state);
    }
    Ok(result)
}
#[command]
pub async fn get_system(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
) -> Result<system_status::Status, ()> {
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
#[command]
pub async fn get_battery(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
) -> Result<Option<battery::Status>, ()> {
    let state = state.lock().await;
    Ok(state.battery.clone())
}
#[command]
pub async fn get_battery_history_page(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
    cursor: Option<i64>,
    size: u8,
) -> Result<Option<Vec<persis::HistoryInfo>>, ()> {
    let now = Utc::now().timestamp();
    let state = state.lock().await;
    match &state.persis {
        Some(persis) => {
            let rows = persis
                .select_history_page(cursor, size, 0, now)
                .await
                .unwrap();
            Ok(Some(rows))
        }
        None => Ok(None),
    }
}
#[command]
pub async fn get_battery_history(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<session::SessionState>>>,
    id: i64,
) -> Result<Option<persis::HistoryInfo>, ()> {
    let state = state.lock().await;
    match &state.persis {
        Some(persis) => {
            let row = persis.get_history(id).await.unwrap();
            Ok(row)
        }
        None => Ok(None),
    }
}
