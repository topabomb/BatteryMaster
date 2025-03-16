mod entities;
pub use entities::*;
mod store;
pub use store::*;
mod down_sample;
mod manager;
pub use manager::*;
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use core::time;

    use sea_orm_migration::prelude::*;
    use status::{Last, Status};
    use std::env;
    async fn get_store() -> BatteryStore {
        dotenv::dotenv().unwrap();
        let store = BatteryStore::new(10, Some(env::var("DATABASE_URL").unwrap()))
            .await
            .unwrap();
        store
    }
    #[tokio::test]
    async fn manager_insert() {
        let now = Utc::now().timestamp();
        let mut manager = manager::Manager::build(&"migration/database/debug.db".to_string(), 10)
            .await
            .unwrap();
        let battery = battery::Status::build().unwrap()[0].clone();
        let system = system::Status::build().unwrap();
        manager
            .insert_battery(&battery, &system, |history_id| async move {
                println!("hit fn({:?})", history_id);
            })
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn select_history_page() {
        let now = Utc::now().timestamp();
        let manager = manager::Manager::build(&"migration/database/debug.db".to_string(), 10)
            .await
            .unwrap();
        let rows = manager
            .select_history_page(None, 30, now - Duration::days(1).num_seconds(), now)
            .await
            .unwrap();
        dbg!(rows);
        ()
    }
    #[tokio::test]
    async fn down_sample() {
        let mut store = get_store().await;
        let mut battery = battery::Status::build().unwrap()[0].clone();
        for _ in 0..1500 {
            battery.last();
            let system = system::Status::build().unwrap();
            let res = store.insert(&battery, &system, |_| async {}).await;
            //在系统进入休眠或睡眠瞬间，insert会Err，需要处理
            match res {
                Ok((_vec,inner)) => {
                    inner.map(|x| {
                        assert_eq!(x.timestamp, battery.timestamp);
                    });
                }
                Err(e) => {
                    dbg!(e);
                }
            }
            tokio::time::sleep(time::Duration::from_secs_f32(0.001)).await;
        }

        let now_millis = Utc::now().timestamp_millis();
        let end = Utc::now().timestamp();
        let r1 = down_sample::down_sample(
            &store.mem_db,
            &down_sample::DownSampleParams {
                table_name: "memory_battery_status".to_string(),
                end_time: end,
                start_time: end - 10,
                interval_secs: 2,
                ..Default::default()
            },
        )
        .await;
        let r2 = r1.as_ref().unwrap();
        dbg!(Utc::now().timestamp_millis() - now_millis);
        dbg!(r2.len());
        assert!(r2.len() == 5);
    }
}
