use std::{fs::create_dir, path::Path};

use sea_orm::DbErr;

use crate::{battery_realtime, store::BatteryStore};
pub struct Manager {
    store: BatteryStore,
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
    pub async fn insert_battery(
        &mut self,
        battery: &battery::Status,
        system: &system::Status,
    ) -> Result<(), DbErr> {
        let _result = self.store.insert(battery, system).await?;
        Ok(())
    }
    pub async fn close(&mut self) {
        self.store.mem_db.close_by_ref().await.unwrap();
        if let Some(db) = &self.store.db {
            db.close_by_ref().await.unwrap();
        }
    }
}
