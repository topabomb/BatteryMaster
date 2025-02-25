use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tauri_plugin_autostart::ManagerExt;
#[cfg(debug_assertions)]
fn check_if_dev() -> bool {
    true
}

#[cfg(not(debug_assertions))]
fn check_if_dev() -> bool {
    false
}

// 定义配置结构体
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Config {
    pub auto_start: bool,             // 系统启动时自动启动
    pub start_minimize: bool,         //启动时最小化
    pub ui_update: u8,                // UI标更新时间
    pub service_update: u8,           // 监控服务更新时间
    pub record_battery_history: bool, // 是否记录电池活动历史
}

impl Default for Config {
    fn default() -> Self {
        Config {
            auto_start: false,
            start_minimize: true,
            ui_update: 2,
            service_update: 1,
            record_battery_history: false,
        }
    }
}

// 获取当前执行目录
pub fn get_exe_directory() -> PathBuf {
    env::current_exe()
        .expect("Failed to get current executable path")
        .parent()
        .unwrap()
        .to_path_buf()
}

// 获取配置文件路径（基于执行文件目录）
pub fn get_config_file_path() -> PathBuf {
    get_exe_directory().join("config.json")
}

// 读取配置文件并返回配置
pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = get_config_file_path();

    if Path::new(&config_path).exists() {
        // 文件存在，读取并解析配置
        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // 解析 JSON 配置
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    } else {
        let config = Config::default();
        save_config(&config).expect("save config err.");
        Ok(config)
    }
}

// 保存配置到文件
pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_file_path();
    let mut file = File::create(config_path)?;

    // 将配置序列化为 JSON
    let config_json = serde_json::to_string_pretty(config)?;
    file.write_all(config_json.as_bytes())?;

    Ok(())
}

pub fn set_autostart(app_handle: &tauri::AppHandle, val: bool) {
    let autostart_manager = app_handle.autolaunch();
    if !check_if_dev() {
        if val {
            if !autostart_manager.is_enabled().unwrap() {
                autostart_manager.enable().expect("autostart enable err.")
            }
        } else {
            if autostart_manager.is_enabled().unwrap() {
                autostart_manager.disable().expect("autostart disable err.")
            }
        }
    }
}
