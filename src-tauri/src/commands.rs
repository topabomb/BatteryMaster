use std::sync::Arc;

use crate::config;
use crate::session;
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
