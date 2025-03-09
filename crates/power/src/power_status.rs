use chrono::prelude::*;
use libapuadj::ryzen_access;
use serde::{Deserialize, Serialize};
use status::{Last, Status as BaseStatus};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Identifier {
    pub cpu_family: i32,
}
impl Default for Identifier {
    fn default() -> Self {
        Self {
            cpu_family: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Status {
    pub identifier: Identifier,
    pub timestamp: i64,
    pub table: i32,
    pub stapm_limit: f32,
    pub stamp_value: f32,
    pub slow_limit: f32,
    pub slow_value: f32,
    pub fast_limit: f32,
    pub fast_value: f32,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            timestamp: Utc::now().timestamp(),
            identifier: Identifier::default(),
            table: 0,
            stapm_limit: 0.0,
            stamp_value: 0.0,
            slow_value: 0.0,
            slow_limit: 0.0,
            fast_limit: 0.0,
            fast_value: 0.0,
        }
    }
}
impl Status {
    pub fn refresh(&mut self, adj: &ryzen_access) {
        if !adj.is_null() {
            self.timestamp = Utc::now().timestamp();
            self.table = unsafe { libapuadj::init_table(*adj) };
            self.stapm_limit =
                (unsafe { libapuadj::get_stapm_limit(*adj) } * 100.0).round() / 100.0;
            self.stamp_value =
                (unsafe { libapuadj::get_stapm_value(*adj) } * 100.0).round() / 100.0;
            self.slow_limit = (unsafe { libapuadj::get_slow_limit(*adj) } * 100.0).round() / 100.0;
            self.slow_value = (unsafe { libapuadj::get_slow_value(*adj) } * 100.0).round() / 100.0;
            self.fast_limit = (unsafe { libapuadj::get_fast_limit(*adj) } * 100.0).round() / 100.0;
            self.fast_value = (unsafe { libapuadj::get_fast_value(*adj) } * 100.0).round() / 100.0;
        }
    }
}
impl BaseStatus<Status> for Status {
    fn build() -> Option<Status> {
        let adj = unsafe { libapuadj::init_ryzenadj() };
        if adj.is_null() {
            None
        } else {
            let mut info = Self::default();
            info.identifier = Identifier {
                cpu_family: unsafe { libapuadj::get_cpu_family(adj) },
            };
            info.refresh(&adj);

            unsafe { libapuadj::cleanup_ryzenadj(adj) };
            Some(info)
        }
    }
}

impl Last for Status {
    fn last(&mut self) {
        let adj = unsafe { libapuadj::init_ryzenadj() };
        if !adj.is_null() {
            self.refresh(&adj);
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct PowerLimit {
    pub stapm_limit: f32,
    pub slow_limit: f32,
    pub fast_limit: f32,
}
impl Default for PowerLimit {
    fn default() -> Self {
        PowerLimit {
            stapm_limit: 0.0,
            slow_limit: 0.0,
            fast_limit: 0.0,
        }
    }
}
pub fn set_limit(limit: &PowerLimit) -> Result<(), Box<dyn std::error::Error>> {
    if limit.fast_limit > 0.0 && limit.slow_limit > 0.0 && limit.stapm_limit > 0.0 {
        let adj = unsafe { libapuadj::init_ryzenadj() };
        if adj.is_null() {
            Err("not support".into())
        } else {
            unsafe {
                libapuadj::set_stapm_limit(adj, (limit.stapm_limit * 1000.0) as u32);
                libapuadj::set_fast_limit(adj, (limit.fast_limit * 1000.0) as u32);
                libapuadj::set_slow_limit(adj, (limit.slow_limit * 1000.0) as u32);
                libapuadj::cleanup_ryzenadj(adj);
            }
            Ok(())
        }
    } else {
        Err("Invalid power limit values".into())
    }
}
