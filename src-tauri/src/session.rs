use crate::battery;
use crate::config;
use crate::power;
use crate::system;

#[derive(Default)]
pub struct SessionState {
    pub config: config::Config,
    pub battery: battery::BatteryInfo,
    pub system: system::SystemInfo,
    pub power_lock: power::PowerLock,
}
