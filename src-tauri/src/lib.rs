use battery::BatteryInfo;
use std::panic;
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
    panic::set_hook(Box::new(|info| {
        // 这里你可以自定义 panic 处理逻辑
        println!("A panic occurred: {:?}", info);

        // 可以选择直接退出进程（如果你希望发生 panic 时退出）
        std::process::exit(1); // 可以替换为你希望的退出代码
    }));
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .setup(|app| {
            let config = config::load_config().expect("load_config err.");
            app.manage(Arc::new(Mutex::new(session::SessionState {
                config: config,
                info: BatteryInfo::default(),
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
                        state.info = info.clone();
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
