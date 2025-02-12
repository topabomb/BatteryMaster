use std::sync::Arc;
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_autostart::ManagerExt;

use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

mod battery;
mod commands; // 引入命令模块
mod config;
mod session;
mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .setup(|app| {
            let config = config::load_config().expect("load_config err.");
            app.manage(Arc::new(Mutex::new(session::SessionState {
                tray_number: 0,
                config: config,
            })));
            config::set_autostart(app.app_handle(), config.auto_start);
            tray::build(app, "main");
            let bms = Arc::new(Mutex::new(battery::Battery::new()));
            battery::Battery::start(bms.clone(), 1);
            let bms_clone = Arc::clone(&bms);
            let handler = app.handle().clone();
            tokio::spawn(async move {
                loop {
                    let mut secs = 1;
                    {
                        let state = handler.state::<Arc<Mutex<session::SessionState>>>();
                        let mut state = state.lock().await;
                        let info = bms_clone.lock().await.current.clone().unwrap();
                        state.tray_number = info.energy_rate.round() as i32;
                        handler.emit("battery_info_updated", info).unwrap();
                        secs = state.config.service_update;
                    }
                    sleep(Duration::from_secs(secs as u64)).await;
                }
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::set_config,
            commands::get_config
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                window.hide().unwrap();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
