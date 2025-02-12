use crate::battery;
use crate::config;

#[derive(Default)]
pub struct SessionState {
    pub config: config::Config,
    pub info: battery::BatteryInfo,
}
