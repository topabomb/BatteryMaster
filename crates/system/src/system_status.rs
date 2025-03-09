use chrono::prelude::*;
use raw_cpuid::CpuId;
use serde::{Deserialize, Serialize};
use status::{Last, Status as BaseStatus};
use wmi::{COMLibrary, WMIConnection};
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct Identifier {
    pub cpu_vendor: String,
    pub cpu_name: String,
    pub mem_total: u64,
    pub hostname: String,
}
impl Default for Identifier {
    fn default() -> Self {
        Self {
            cpu_vendor: Default::default(),
            cpu_name: Default::default(),
            mem_total: 0,
            hostname: Default::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Status {
    //额外的标识信息
    identifier: Identifier,
    //时间戳
    pub timestamp: i64,
    //是否支持cpu功耗限制
    pub support_power_set: bool,
    //cpu占用
    pub cpuload: f32,
    //空闲内存
    pub memfree: u32,
    //屏幕亮度
    pub screen_brightness: f32,
    //屏幕标识
    pub screen_instance: String,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            support_power_set: false,
            identifier: Default::default(),
            timestamp: Default::default(),
            cpuload: Default::default(),
            memfree: Default::default(),
            screen_brightness: Default::default(),
            screen_instance: String::from("Unknown"),
        }
    }
}
#[derive(Deserialize, Debug)]
struct WmiMonitorBrightness {
    Active: bool,
    CurrentBrightness: u8,
    InstanceName: String,
}
impl Status {
    fn refresh_brightness(&mut self) {
        let com_con = COMLibrary::new();
        let com_con = match com_con {
            Ok(v) => v,
            Err(e) => match e {
                wmi::WMIError::HResultError { hres } => {
                    if hres == -2147417850 {
                        unsafe { COMLibrary::assume_initialized() }
                    } else {
                        panic!("wmi::WMIError::HResultError({hres}):{e}")
                    }
                }
                _ => panic!("{e}"),
            },
        };
        let results: Vec<WmiMonitorBrightness> =
            WMIConnection::with_namespace_path("root\\WMI", com_con)
                .expect("not connect WMI")
                .raw_query("SELECT Active,CurrentBrightness,InstanceName FROM WmiMonitorBrightness")
                .expect("WmiMonitorBrightness err.");
        let curr = results.iter().find(|x| x.Active);
        if curr.is_some() {
            let curr = curr.unwrap();
            self.screen_brightness = curr.CurrentBrightness as f32 / 100.0;
            self.screen_instance = curr.InstanceName.clone();
        }
    }
    fn refresh(&mut self) {
        self.timestamp = Utc::now().timestamp();
        self.cpuload = match sys_info::loadavg() {
            Ok(v) => v.one as f32,
            Err(_) => 0.0,
        };
        match sys_info::mem_info() {
            Ok(v) => self.memfree = v.free as u32,
            Err(_) => (),
        };
        self.refresh_brightness();
    }
}

impl BaseStatus<Status> for Status {
    fn build() -> Option<Self> {
        let mut info = Self::default();
        let cpu: CpuId<raw_cpuid::CpuIdReaderNative> = CpuId::new();
        info.identifier = Identifier {
            cpu_name: cpu
                .get_processor_brand_string()
                .as_ref()
                .map_or_else(|| "Unknown", |pbs| pbs.as_str())
                .to_string(),
            cpu_vendor: cpu
                .get_vendor_info()
                .as_ref()
                .map_or_else(|| "Unknown", |pbs| pbs.as_str())
                .to_string(),
            mem_total: match sys_info::mem_info() {
                Ok(v) => v.total,
                Err(_) => 0,
            },
            hostname: match sys_info::hostname() {
                Ok(v) => v.to_string(),
                Err(_) => "Unknown".to_string(),
            },
        };
        info.support_power_set = match info.identifier.cpu_vendor.as_str() {
            "AuthenticAMD" => true,
            _ => false,
        };
        info.refresh();

        Some(info)
    }
}
impl Last for Status {
    fn last(&mut self) {
        self.refresh();
    }
}
