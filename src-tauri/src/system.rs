use raw_cpuid::CpuId;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    pub cpu_vendor: String,
    pub cpu_name: String,
    pub support_power_set: bool,
}
impl Default for SystemInfo {
    fn default() -> Self {
        SystemInfo {
            cpu_vendor: "unknow".to_string(),
            cpu_name: "unknow".to_string(),
            support_power_set: false,
        }
    }
}
impl SystemInfo {
    pub fn new() -> Self {
        let mut info = Self::default();
        let cpu: CpuId<raw_cpuid::CpuIdReaderNative> = CpuId::new();
        info.cpu_name = cpu
            .get_processor_brand_string()
            .as_ref()
            .map_or_else(|| "unknow", |pbs| pbs.as_str())
            .to_string();
        info.cpu_vendor = cpu
            .get_vendor_info()
            .as_ref()
            .map_or_else(|| "unknow", |pbs| pbs.as_str())
            .to_string();
        info.support_power_set = match info.cpu_vendor.as_str() {
            "AuthenticAMD" => true,
            _ => false,
        };
        info
    }
}
