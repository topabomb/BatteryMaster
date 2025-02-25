use log::{log, Level};
use std::sync::Arc;
use std::time::SystemTime;
use std::{env, panic};
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tokio::sync::{oneshot, Mutex};
use tokio::task;
use tokio::time::{sleep, Duration};
mod battery;
mod commands;
mod config;
mod power;
mod session;
mod system;
mod tray;
mod windows;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    const TRAY_ID: &str = "main";
    let args: Vec<String> = env::args().collect();
    let is_autostart = args.contains(&"--autostart".to_string());
    let is_adminstart = args.contains(&"--adminstart".to_string());

    panic::set_hook(Box::new(|info| {
        log!(Level::Error, "A panic occurred: {:?}", info);
        std::process::exit(1); // 可以替换为你希望的退出代码
    }));
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Folder {
                        path: config::get_exe_directory(),
                        file_name: Some("logs".to_string()),
                    },
                ))
                .max_file_size(1_000_000 /* bytes */)
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            windows::active_window_change_state(app, "main");
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .setup(move |app| {
            log!(Level::Debug, "args ={:?}", args);

            let config = config::load_config().expect("load_config err.");
            app.manage(Arc::new(Mutex::new(session::SessionState::new(config))));
            if is_adminstart {
                windows::active_window(app.handle(), "main");
            }

            config::set_autostart(app.app_handle(), config.auto_start);
            tray::build(app, TRAY_ID);
            if config.start_minimize {
                if let Some(window) = app.get_webview_window("main") {
                    window.close().unwrap();
                }
            }
            let bms = Arc::new(Mutex::new(battery::Battery::new()));
            battery::Battery::start(bms.clone(), 1);
            let bms_clone = Arc::clone(&bms);
            let handler1 = app.handle().clone();
            let handler2 = app.handle().clone();
            let (tx, rx) = oneshot::channel::<()>();
            //service update.
            tokio::spawn(async move {
                let mut tx = Some(tx); // 将 tx 包装成 Option，以便在第一次发送后取出
                loop {
                    let mut secs = 1;
                    {
                        let state = handler1.state::<Arc<Mutex<session::SessionState>>>();
                        let mut state = state.lock().await;
                        let info = bms_clone.lock().await.current.clone().unwrap();
                        //battery
                        state.battery = info.clone();
                        //power
                        if state.is_admin && state.system.support_power_set {
                            state.power = match power::PowerInfo::new() {
                                Ok(v) => v,
                                Err(_) => {
                                    log!(Level::Warn, "loop power_lock get_powerinfo err.");
                                    state.system.support_power_set = false;
                                    power::PowerInfo::default()
                                }
                            };
                        }
                        //power_lock
                        if state.power_lock.enable {
                            let now = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            if now - state.power_lock.lastcheck > 10
                                && state.is_admin
                                && state.system.support_power_set
                            {
                                state.power_lock.lastcheck = now;
                                let info = power::PowerInfo::new();
                                if let Ok(val) = info {
                                    let info = val;
                                    let limit = state.power_lock.limit.clone();
                                    if info.fast_limit != limit.fast_limit
                                        || info.slow_limit != limit.slow_limit
                                        || info.stapm_limit != limit.stapm_limit
                                    {
                                        match power::set_limit(&state.power_lock.limit) {
                                            Ok(_) => {
                                                let info = power::PowerInfo::new().unwrap();
                                                log!(Level::Debug, "in loop,set_limit:{:?}", info);
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
                        session::EventChannel::emit_service_update(&handler1, &state).await;
                        secs = state.config.service_update;
                    }
                    if let Some(t) = tx.take() {
                        let _ = t.send(()).map_err(|_| ());
                    }
                    sleep(Duration::from_secs(secs as u64)).await;
                }
            });
            //ui update.
            tokio::spawn(async move {
                let mut rx = Some(rx); // 将 tx 包装成 Option，以便在第一次发送后取出
                loop {
                    if let Some(rx) = rx.take() {
                        let _ = rx.await;
                    }
                    let tary = handler2.tray_by_id(TRAY_ID).unwrap();
                    tray::update_tray_icon(&tary).await;
                    let mut secs = 1;
                    {
                        let state = handler2.state::<Arc<Mutex<session::SessionState>>>();
                        let state = state.lock().await;
                        secs = state.config.ui_update;

                        session::EventChannel::emit_ui_update(&handler2, &state).await;
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
            commands::set_event_channel,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                window.hide().unwrap();

                let handler = window.app_handle();
                let state: tauri::State<'_, Arc<Mutex<session::SessionState>>> =
                    handler.state::<Arc<Mutex<session::SessionState>>>();

                // 将 state 转移到异步任务中
                tokio::spawn({
                    let state = Arc::clone(&state); // 克隆 Arc 以便传递给异步任务
                    async move {
                        let mut state = state.lock().await; // 异步地获取 Mutex 锁
                        state.is_min_tray = true; // 修改状态
                    }
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
