use std::collections::HashMap;
use std::fs::{self, File};
use std::path::PathBuf;
use std::{fmt, path::Path};

use crate::battery::{BatteryInfo, SerializableState};
use crate::config;
use crate::dataframe::IntoDataFrame;
use crate::power::PowerInfo;
use crate::session::SessionState;
use chrono::{NaiveDateTime, Utc};
use log::{log, Level};
use polars::frame::DataFrame;
use polars::prelude::*;
use polars::time::*;
#[derive(Debug)]
struct DataFrameCache {
    file_df: DataFrame,
    cache_df: DataFrame,
}
#[derive(Clone, Debug)]
enum HistoryType {
    Battery,
    Power,
}
impl fmt::Display for HistoryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HistoryType::Battery => write!(f, "battery"),
            HistoryType::Power => write!(f, "power"),
        }
    }
}
#[derive(Clone, Debug)]
struct IntervalConfig {
    htype: HistoryType,
    interval: String,
    limit: usize,
}
impl IntervalConfig {
    fn dir(&self) -> String {
        config::get_exe_directory()
            .join("data")
            .to_str()
            .unwrap()
            .to_string()
    }
    fn filename(&self) -> String {
        String::from(format!("{}_{}.parquet", self.htype, self.interval))
    }
    pub fn filepath(&self) -> PathBuf {
        PathBuf::from(self.dir().as_str()).join(self.filename())
    }
}
#[derive(Debug)]
pub struct DataProcessor {
    types: Vec<IntervalConfig>,
    last_update_at: i64,
    cache: HashMap<String, DataFrameCache>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let types = vec![
            IntervalConfig {
                htype: HistoryType::Battery,
                interval: "10s".to_string(),
                limit: 360,
            },
            IntervalConfig {
                htype: HistoryType::Power,
                interval: "10s".to_string(),
                limit: 360,
            },
        ];
        let mut cache = HashMap::<String, DataFrameCache>::new();
        for t in types.iter() {
            if !Path::new(&t.dir()).exists() {
                fs::create_dir(&t.dir()).unwrap();
            }
            let mut file_df;
            let cache_df = match t.htype {
                HistoryType::Battery => BatteryInfo::empty_data_frame(),
                HistoryType::Power => PowerInfo::empty_data_frame(),
            };
            if (Path::new(&t.filepath())).exists() {
                file_df = LazyFrame::scan_parquet(&t.filepath(), ScanArgsParquet::default())
                    .expect(format!("open file {:?} err.", &t.filepath()).as_str())
                    .collect()
                    .expect(format!("collect file {:?} err.", &t.filepath()).as_str());
                cache.insert(t.filename(), DataFrameCache { file_df, cache_df });
            } else {
                cache.insert(
                    t.filename(),
                    DataFrameCache {
                        file_df: cache_df.clone(),
                        cache_df,
                    },
                );
            }
        }
        Self {
            types,
            last_update_at: Utc::now().timestamp(),
            cache,
        }
    }
    pub fn push_cache<T>(&mut self, h: &IntervalConfig, data: T)
    where
        T: IntoDataFrame,
    {
        match self.cache.get_mut(&h.filename()) {
            Some(v) => {
                let height = v.cache_df.height();
                if height > h.limit {
                    v.cache_df = v.cache_df.slice((height - h.limit) as i64, height);
                }
                v.cache_df.vstack_mut(&data.into_data_frame()).unwrap();
            }
            None => (),
        };
    }
    pub fn merge_and_save(&mut self) {
        let types = self.types.clone();
        for v in &types {
            let cache = &mut self.cache.get_mut(&v.filename()).unwrap();
            let grouped = cache
                .cache_df
                .clone()
                .lazy()
                .with_columns(vec![
                    // 创建一列标识当前行的状态变化
                    col("state").shift(lit(1)).alias("prev_state"), // 获取上一行的状态
                ])
                .filter(
                    // 在每个时间窗口内，检查状态是否变化
                    col("state")
                        .neq(col("prev_state"))
                        .or(col("prev_state").is_null()),
                )
                .group_by_dynamic(
                    col("timestamp"),
                    [],
                    DynamicGroupOptions {
                        every: Duration::parse(&v.interval),
                        period: Duration::parse(&v.interval),
                        offset: Duration::parse("0s"),
                        ..Default::default()
                    },
                );
            let grouped = match v.htype {
                HistoryType::Battery => grouped.agg([
                    col("percentage").mean(),
                    col("energy_rate").mean(),
                    col("voltage").mean(),
                    col("state_of_health").mean(),
                    col("state").last(),
                ]),
                HistoryType::Power => grouped.agg([
                    col("stapm_limit").mean(),
                    col("stamp_value").mean(),
                    col("slow_limit").mean(),
                    col("slow_value").mean(),
                    col("fast_limit").mean(),
                    col("fast_value").mean(),
                ]),
            };
            let grouped = grouped.collect();
            if let Err(e) = grouped {
                log!(Level::Error, "err:{:?}", e);
                return;
            }
            //print!("{:?}", grouped);
            cache.file_df.vstack_mut(&grouped.unwrap()).unwrap();
            cache.cache_df.clear();

            let height = cache.file_df.height();
            if height > v.limit {
                cache.file_df = cache.file_df.slice((height - v.limit) as i64, height);
            }

            let mut file = File::create(v.filepath()).unwrap();
            ParquetWriter::new(&mut file)
                .finish(&mut cache.file_df)
                .unwrap();
            println!("{:?}", cache.file_df);
        }
    }
    pub fn update(&mut self, state: &SessionState) {
        let now = Utc::now().timestamp();
        let types = self.types.clone();
        for v in &types {
            match v.htype {
                HistoryType::Battery => {
                    if state.battery.state != SerializableState(battery::State::Unknown) {
                        self.push_cache(&v, state.battery.clone())
                    }
                }
                HistoryType::Power => {
                    if state.power.cpu_family > 0 {
                        self.push_cache(&v, state.power.clone())
                    }
                }
            };
        }
        if now - self.last_update_at > 10 {
            self.merge_and_save();
            self.last_update_at = now;
        }
    }
}
