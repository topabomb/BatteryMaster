use chrono::prelude::*;
use log::{log, Level};
use status::Last;
use std::sync::Arc;
use std::{env, panic};
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tokio::sync::{oneshot, Mutex};
use tokio::time::{sleep, Duration};
mod commands;
mod config;
mod session;
mod tray;
mod windows;
//mod processor;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    const TRAY_ID: &str = "main";
    let args: Vec<String> = env::args().collect();
    let is_autostart = args.contains(&"--autostart".to_string());
    let is_adminstart = args.contains(&"--adminstart".to_string());
    panic::set_hook(Box::new(|info| {
        let payload = info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic payload"
        };
        // 提取位置信息
        let location = info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        log!(
            Level::Error,
            "A panic occurred: message='{}', location='{}'",
            message,
            location
        );
        std::process::exit(1); // 可以替换为你希望的退出代码
    }));
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Folder {
                        path: config::get_exe_directory(),
                        file_name: Some("logs".to_string()),
                    },
                ))
                .max_file_size(1_000_000 /* bytes */)
                .level(log::LevelFilter::Warn)
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseUtc)
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            windows::active_window(app, "main");
            let state = app.state::<Arc<Mutex<session::SessionState>>>();
            tokio::spawn({
                let state = Arc::clone(&state);
                async move {
                    let mut state = state.lock().await; // 异步地获取 Mutex 锁
                    state.is_min_tray = false; // 修改状态
                }
            });
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .setup(move |app| {
            log!(Level::Info, "args ={:?}", args);
            let config = config::load_config().expect("load_config err.");
            let session = session::SessionState::new(config);
            app.manage(Arc::new(Mutex::new(session)));
            if is_adminstart {
                windows::active_window(app.handle(), "main");
            }
            config::set_autostart(app.app_handle(), config.auto_start);
            tray::build(app, TRAY_ID);
            if config.start_minimize && !is_adminstart {
                if let Some(window) = app.get_webview_window("main") {
                    window.close().unwrap();
                }
            }

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
                        //system
                        state.system.last();
                        //battery
                        if state.battery.is_some() {
                            let battery = state.battery.as_mut().unwrap();
                            let last_state = battery.state.clone();
                            battery.last();
                            if battery.state_changed {
                                log!(
                                    Level::Warn,
                                    "Battery State {:?}->{:?}",
                                    last_state,
                                    battery.state
                                );
                            }
                        }
                        //power
                        if state.is_admin && state.system.support_power_set {
                            if state.power.is_some() {
                                state.power.as_mut().unwrap().last();
                            } else {
                                log!(Level::Warn, "loop get_powerinfo err.");
                                state.system.support_power_set = false;
                            }
                        }
                        //store
                        if state.config.record_battery_history && state.battery.is_some() {
                            if state.persis.is_none() {
                                state.persis = Some(
                                    persis::Manager::build(
                                        &config::get_exe_directory()
                                            .join("history.db")
                                            .to_str()
                                            .unwrap()
                                            .to_string(),
                                        10,
                                    )
                                    .await
                                    .unwrap(),
                                );
                            }
                        } else {
                            if let Some(ref mut manager) = state.persis {
                                manager.close().await;
                                state.persis = None;
                            }
                        }
                        let battery = state.battery.clone().unwrap();
                        let system = state.system.clone();
                        if let Some(manager) = &mut state.persis {
                            if battery.state_changed {
                                log!(
                                    Level::Warn,
                                    "process persis at({}),Battery State changed to {:?}",
                                    battery.timestamp,
                                    battery.state
                                );
                            }
                            let res = manager
                                .insert_battery(&battery, &system, |_| async {
                                    log!(Level::Warn, "new battery history ");
                                    /*绝对不能在lock中再次lock，会导致死锁
                                    let state =
                                        handler1.state::<Arc<Mutex<session::SessionState>>>();
                                    let state = state.lock().await;
                                    session::EventChannel::emit_history_update(&handler1, &state);
                                    */
                                })
                                .await;
                            match res {
                                Ok(res) => {
                                    if res.contains(&persis::InsertModifyed::BatteryHistory) {
                                        session::EventChannel::emit_history_update(
                                            &handler1, &state,
                                        );
                                    }
                                }
                                Err(e) => {
                                    log!(Level::Error, "manager.insert_battery error:{}", e);
                                }
                            }
                        }
                        //power_lock
                        if state.power_lock.enable
                            && state.is_admin
                            && state.system.support_power_set
                        {
                            let now = Utc::now().timestamp();
                            if now - state.power_lock.lastcheck > 10 && state.power.is_some() {
                                state.power_lock.lastcheck = now;
                                let limit = state.power_lock.limit.clone();
                                if let Some(info) = state.power.as_mut() {
                                    info.last();
                                    if info.fast_limit != limit.fast_limit
                                        || info.slow_limit != limit.slow_limit
                                        || info.stapm_limit != limit.stapm_limit
                                    {
                                        match power::set_limit(&limit) {
                                            Ok(_) => {
                                                {
                                                    info.last();
                                                }
                                                log!(Level::Warn, "in loop,set_limit:{:?}", info);
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
                        //store
                        //processor.update(&state);
                        session::EventChannel::emit_service_update(&handler1, &state);
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
                let mut rx = Some(rx);
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

                        session::EventChannel::emit_ui_update(&handler2, &state);
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
            commands::get_battery,
            commands::get_battery_history_page,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                window.hide().unwrap();

                let handler = window.app_handle();
                let state: tauri::State<'_, Arc<Mutex<session::SessionState>>> =
                    handler.state::<Arc<Mutex<session::SessionState>>>();
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
