use battery::BatteryInfo;
use log::{log, Level};
use std::panic;
use std::sync::Arc;
use std::time::SystemTime;
use system::SystemInfo;
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
mod battery;
mod commands; // 引入命令模块
mod config;
mod power;
mod session;
mod system;
mod tray;
mod windows;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    panic::set_hook(Box::new(|info| {
        // 这里你可以自定义 panic 处理逻辑
        println!("A panic occurred: {:?}", info);
        // 可以选择直接退出进程（如果你希望发生 panic 时退出）
        std::process::exit(1); // 可以替换为你希望的退出代码
    }));
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Folder {
                        path: config::get_exe_directory(),
                        file_name: Some("BatteryMonitor".to_string()),
                    },
                ))
                .max_file_size(1_000_000 /* bytes */)
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            windows::active_window(app, "main");
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .setup(|app| {
            let config = config::load_config().expect("load_config err.");
            app.manage(Arc::new(Mutex::new(session::SessionState {
                config,
                battery: BatteryInfo::default(),
                system: SystemInfo::new(),
                power_lock: power::PowerLock::default(),
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
                        state.battery = info.clone();
                        handler.emit("battery_info_updated", info).unwrap();
                        secs = state.config.service_update;

                        if state.power_lock.enable {
                            let now = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            if now - state.power_lock.lastcheck > 10
                                && windows::is_admin()
                                && state.system.support_power_set
                            {
                                state.power_lock.lastcheck = now;
                                let info = power::PowerInfo::new();
                                if let Ok(val) = info {
                                    let info = val;
                                    handler.emit("power_info_updated", &info).unwrap();
                                    let limit = state.power_lock.limit.clone();
                                    if info.fast_limit != limit.fast_limit
                                        || info.slow_limit != limit.slow_limit
                                        || info.stapm_limit != limit.stapm_limit
                                    {
                                        match power::set_limit(&state.power_lock.limit) {
                                            Ok(_) => {
                                                let info = power::PowerInfo::new().unwrap();
                                                log!(Level::Debug, "in loop,set_limit:{:?}", info);
                                                handler.emit("power_info_updated", info).unwrap();
                                            }
                                            Err(err) => {
                                                state.power_lock.enable = false;
                                                log!(
                                                    Level::Error,
                                                    "in loop,set_limit err:{:?}",
                                                    err
                                                );
                                            }
                                        }
                                    }
                                } else {
                                    log!(Level::Warn, "loop power_lock get_powerinfo err.");
                                    state.system.support_power_set = false;
                                }
                            }
                        }
                    }
                    sleep(Duration::from_secs(secs as u64)).await;
                }
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::set_config,
            commands::get_config,
            commands::get_isadmin,
            commands::get_powerinfo,
            commands::exec_elevate_self,
            commands::set_power_limit,
            commands::set_power_limit_lock,
            commands::get_system,
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
