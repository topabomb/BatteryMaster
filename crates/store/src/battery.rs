use crate::dataframe;
use battery::{
    units::{electric_potential::volt, energy::watt_hour, power::watt, ratio::ratio, time::second},
    Manager as BatteryManager, State,
};
use chrono::prelude::*;
use polars::{frame::DataFrame, prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sys_info::loadavg;
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct SerializableState(pub State);
impl Serialize for SerializableState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let state_str = match self.0 {
            State::Unknown => "Unknown",
            State::Charging => "Charging",
            State::Discharging => "Discharging",
            State::Empty => "Empty",
            State::Full => "Full",
            State::__Nonexhaustive => "Nonexhaustive",
        };
        serializer.serialize_str(state_str)
    }
}
impl<'de> Deserialize<'de> for SerializableState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let state_str = String::deserialize(deserializer)?;

        let state = match state_str.as_str() {
            "Unknown" => State::Unknown,
            "Charging" => State::Charging,
            "Discharging" => State::Discharging,
            "Empty" => State::Empty,
            "Full" => State::Full,
            _ => {
                return Err(serde::de::Error::unknown_variant(
                    &state_str,
                    &["Unknown", "Charging", "Discharging", "Empty", "Full"],
                ))
            }
        };

        Ok(SerializableState(state))
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BatteryInfo {
    pub state_changed: bool,
    //时间戳
    pub timestamp: i64,
    //状态
    pub state: SerializableState,
    //电量百分比
    pub percentage: f32,
    //充放电瓦数
    pub energy_rate: f32,
    //电池电压
    pub voltage: f32,
    //电池健康状态
    pub state_of_health: f32,
    //设计容量
    pub design_capacity: f32,
    //满充容量
    pub full_capacity: f32,
    //当前容量
    pub capacity: f32,
    //cpu使用率
    pub cpu_load: f32,

    pub serial_number: String,
    pub time_to_empty_secs: u64,
    pub time_to_full_secs: u64,
}
impl Default for BatteryInfo {
    fn default() -> Self {
        BatteryInfo {
            state_changed: false,
            timestamp: Utc::now().timestamp(),
            state: SerializableState(State::default()),
            percentage: 0.0,
            energy_rate: 0.0,
            voltage: 0.0,
            state_of_health: 0.0,
            design_capacity: 0.0,
            full_capacity: 0.0,
            capacity: 0.0,
            cpu_load: 0.0,
            serial_number: String::from("unknow"),
            time_to_empty_secs: 0,
            time_to_full_secs: 0,
        }
    }
}
impl BatteryInfo {
    pub fn new() -> Self {
        Self::default()
    }
}
fn into_data_frame(data: Option<&BatteryInfo>) -> DataFrame {
    let (info, valid) = match data {
        Some(v) => (v, true),
        None => (&BatteryInfo::default(), false),
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
            PlSmallStr::from_str("percentage"),
            match valid {
                true => vec![info.percentage],
                false => vec![] as Vec<f32>,
            },
        ),
        Column::new(
            PlSmallStr::from_str("energy_rate"),
            match valid {
                true => vec![info.energy_rate],
                false => vec![] as Vec<f32>,
            },
        ),
        Column::new(
            PlSmallStr::from_str("voltage"),
            match valid {
                true => vec![info.voltage],
                false => vec![] as Vec<f32>,
            },
        ),
        Column::new(
            PlSmallStr::from_str("state_of_health"),
            match valid {
                true => vec![info.state_of_health],
                false => vec![] as Vec<f32>,
            },
        ),
        Column::new(
            PlSmallStr::from_str("state"),
            match valid {
                true => vec![serde_json::to_string(&info.state).unwrap()],
                false => vec![] as Vec<String>,
            },
        ),
    ])
    .unwrap()
}

impl dataframe::IntoDataFrame for BatteryInfo {
    fn into_data_frame(&self) -> DataFrame {
        into_data_frame(Some(self))
    }

    fn empty_data_frame() -> DataFrame {
        into_data_frame(None)
    }
}
#[derive(Clone)]
pub struct Battery {
    pub prev: Option<BatteryInfo>,
}
impl Battery {
    pub fn new() -> Battery {
        Battery { prev: None }
    }
    pub fn last(&mut self) -> BatteryInfo {
        let bms = BatteryManager::new().unwrap();
        let cpuid = raw_cpuid::CpuId::new();
        let cpu_model = cpuid
            .get_processor_brand_string()
            .as_ref()
            .map_or_else(|| "nan", |pbs| pbs.as_str())
            .to_string();
        let mut record = BatteryInfo::default();

        //查询电池数量
        let batteries = bms.batteries().unwrap();
        for battery in batteries {
            let battery = battery.unwrap();
            record.serial_number = match battery.serial_number() {
                Some(val) => String::from(val),
                None => String::from("unknow"),
            };
            record.percentage = battery.state_of_charge().get::<ratio>();
            record.state = SerializableState(battery.state());
            record.voltage = battery.voltage().get::<volt>();
            record.state_of_health = battery.state_of_health().get::<ratio>();
            record.energy_rate = match record.state {
                SerializableState(State::Charging) => battery.energy_rate().get::<watt>(),
                SerializableState(State::Discharging) => -battery.energy_rate().get::<watt>(),
                _ => 0.0,
            };
            record.design_capacity = battery.energy_full_design().get::<watt_hour>();
            record.full_capacity = battery.energy_full().get::<watt_hour>();
            record.capacity = battery.energy().get::<watt_hour>();
            record.cpu_load = loadavg().unwrap().one as f32;

            record.time_to_empty_secs = match battery.time_to_empty() {
                Some(duration) => duration.get::<second>() as u64,
                None => 0,
            };
            record.time_to_full_secs = match battery.time_to_full() {
                Some(duration) => duration.get::<second>() as u64,
                None => 0,
            };
            break;
        }
        if let Some(v) = &self.prev {
            if v.state != record.state && v.state != SerializableState(::battery::State::Unknown) {
                record.state_changed = true;
            }
        }
        self.prev = Some(record.clone());
        record
    }
}
