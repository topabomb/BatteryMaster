use std::{fs::create_dir, path::Path};

use crate::{entities::*, store};
use sea_orm::{DbErr, FromQueryResult, Statement};
use serde::*;

use crate::{battery_realtime, store::BatteryStore};
pub struct Manager {
    store: BatteryStore,
}
#[derive(Serialize, Deserialize, Clone, Debug, FromQueryResult)]
pub struct HistoryInfo {
    prev_timestamp: Option<i64>,
    timestamp_diff: Option<i64>,

    prev_state_of_health: Option<f32>,
    state_of_health_diff: Option<f32>,

    prev_percentage: Option<f32>,
    percentage_diff: Option<f32>,

    prev_capacity: Option<f32>,
    capacity_diff: Option<f32>,

    timestamp: i64,
    end_at: Option<i64>,
    state: String,
    prev: Option<String>,

    capacity: f32,
    full_capacity: f32,
    design_capacity: f32,

    percentage: f32,
    state_of_health: f32,
    energy_rate: f32,
    voltage: f32,
    cpu_load: f32,
}
impl Manager {
    pub async fn build(db_path: &String, interval_secs: u32) -> Result<Self, String> {
        let path = Path::new(db_path);
        if let Some(dir) = path.parent() {
            if !dir.exists() {
                create_dir(dir).unwrap();
            }
        }
        let conn_str = format!("sqlite://{}?mode=rwc", path.to_str().unwrap());
        let store = BatteryStore::new(interval_secs, Some(conn_str)).await;
        match store {
            Ok(v) => Ok(Self { store: v }),
            Err(e) => Err(e.to_string()),
        }
    }
    pub async fn insert_battery<F>(
        &mut self,
        battery: &battery::Status,
        system: &system::Status,
        f: F,
    ) -> Result<Vec<store::InsertModifyed>, DbErr>
    where
        F: AsyncFnOnce(i64) -> (),
    {
        let (vec, _model) = self.store.insert(battery, system, f).await?;
        Ok(vec)
    }
    pub async fn close(&mut self) {
        self.store.mem_db.close_by_ref().await.unwrap();
        if let Some(db) = &self.store.db {
            db.close_by_ref().await.unwrap();
        }
    }
    pub async fn select_history_page(
        &self,
        cursor: Option<i64>,
        size: u8,
        start: i64,
        end: i64,
    ) -> Result<Vec<HistoryInfo>, DbErr> {
        let db = self.store.db.as_ref().unwrap();
        let rows = HistoryInfo::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            r#"
SELECT 
    LAG("timestamp") OVER (
	ORDER BY "timestamp") AS prev_timestamp,
    "timestamp" - LAG("prev_timestamp") OVER (
	ORDER BY "timestamp") AS timestamp_diff,
	LAG("state_of_health") OVER (
	ORDER BY "timestamp") AS prev_state_of_health,
	"state_of_health" - LAG("state_of_health") OVER (
	ORDER BY "timestamp") AS state_of_health_diff,
	LAG("capacity") OVER (
	ORDER BY "timestamp") AS prev_capacity,
	"capacity" - LAG("capacity") OVER (
	ORDER BY "timestamp") AS capacity_diff,
	LAG("percentage") OVER (
	ORDER BY "timestamp") AS prev_percentage,
	"percentage" - LAG("percentage") OVER (
	ORDER BY "timestamp") AS percentage_diff,
	*
FROM
	"battery_state_history"
	WHERE "timestamp" BETWEEN $1 AND $2
    AND "timestamp"<$3
ORDER BY
	"timestamp" DESC
LIMIT $4
;
        "#,
            [
                start.into(),
                end.into(),
                match cursor {
                    Some(v) => v.into(),
                    None => end.into(),
                },
                size.into(),
            ],
        ))
        .all(db)
        .await;
        rows
    }
}
