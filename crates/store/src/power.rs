use chrono::prelude::*;

use polars::{frame::DataFrame, prelude::*};
use raw_cpuid::CpuId;
use serde::{Deserialize, Serialize};

use crate::dataframe;
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

pub struct PowerLock {
    pub limit: PowerLimit,
    pub enable: bool,
    pub lastcheck: i64,
}
impl Default for PowerLock {
    fn default() -> Self {
        Self {
            limit: Default::default(),
            enable: false,
            lastcheck: Utc::now().timestamp(),
        }
    }
}
impl PowerLock {
    pub fn new() -> Self {
        PowerLock::default()
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PowerInfo {
    pub timestamp: i64,
    pub table: i32,
    pub cpu_family: i32,
    pub stapm_limit: f32,
    pub stamp_value: f32,
    pub slow_limit: f32,
    pub slow_value: f32,
    pub fast_limit: f32,
    pub fast_value: f32,
}

impl Default for PowerInfo {
    fn default() -> Self {
        PowerInfo {
            timestamp: Utc::now().timestamp(),
            table: 0,
            cpu_family: 0,
            stapm_limit: 0.0,
            stamp_value: 0.0,
            slow_value: 0.0,
            slow_limit: 0.0,
            fast_limit: 0.0,
            fast_value: 0.0,
        }
    }
}
fn into_data_frame(data: Option<&PowerInfo>) -> DataFrame {
    let (info, valid) = match data {
        Some(v) => (v, true),
        None => (&PowerInfo::default(), false),
    };

    DataFrame::new(vec![
        Column::new(
            PlSmallStr::from_str("timestamp"),
            match valid {
                true => vec![NaiveDateTime::from_timestamp(info.timestamp, 0)],
                false => vec![] as Vec<NaiveDateTime>,
            },
        ),
        Column::new(
            PlSmallStr::from_str("stapm_limit"),
            match valid {
                true => vec![info.stapm_limit],
                false => vec![] as Vec<f32>,
            },
        ),
        Column::new(
            PlSmallStr::from_str("stamp_value"),
            match valid {
                true => vec![info.stamp_value],
                false => vec![] as Vec<f32>,
            },
        ),
        Column::new(
            PlSmallStr::from_str("slow_limit"),
            match valid {
                true => vec![info.slow_limit],
                false => vec![] as Vec<f32>,
            },
        ),
        Column::new(
            PlSmallStr::from_str("slow_value"),
            match valid {
                true => vec![info.slow_value],
                false => vec![] as Vec<f32>,
            },
        ),
        Column::new(
            PlSmallStr::from_str("fast_limit"),
            match valid {
                true => vec![info.fast_limit],
                false => vec![] as Vec<f32>,
            },
        ),
        Column::new(
            PlSmallStr::from_str("fast_value"),
            match valid {
                true => vec![info.fast_value],
                false => vec![] as Vec<f32>,
            },
        ),
    ])
    .unwrap()
}
impl dataframe::IntoDataFrame for PowerInfo {
    fn into_data_frame(&self) -> DataFrame {
        into_data_frame(Some(self))
    }

    fn empty_data_frame() -> DataFrame {
        into_data_frame(None)
    }
}
impl PowerInfo {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let adj = unsafe { libapuadj::init_ryzenadj() };
        if adj.is_null() {
            Err("not support".into())
        } else {
            let mut info = Self::default();

            info.table = unsafe { libapuadj::init_table(adj) };
            info.cpu_family = unsafe { libapuadj::get_cpu_family(adj) };
            info.stapm_limit = (unsafe { libapuadj::get_stapm_limit(adj) } * 100.0).round() / 100.0;
            info.stamp_value = (unsafe { libapuadj::get_stapm_value(adj) } * 100.0).round() / 100.0;
            info.slow_limit = (unsafe { libapuadj::get_slow_limit(adj) } * 100.0).round() / 100.0;
            info.slow_value = (unsafe { libapuadj::get_slow_value(adj) } * 100.0).round() / 100.0;
            info.fast_limit = (unsafe { libapuadj::get_fast_limit(adj) } * 100.0).round() / 100.0;
            info.fast_value = (unsafe { libapuadj::get_fast_value(adj) } * 100.0).round() / 100.0;

            unsafe { libapuadj::cleanup_ryzenadj(adj) };
            Ok(info)
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
