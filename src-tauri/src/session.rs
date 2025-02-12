use crate::config;

#[derive(Default)]
pub struct SessionState {
    pub tray_number: i32,
    pub config: config::Config,
}
