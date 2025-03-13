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
    use chrono::Utc;
    use core::time;

    use sea_orm_migration::prelude::*;
    use status::{Last, Status};
    use std::{env, time::Duration};
    async fn get_store() -> BatteryStore {
        dotenv::dotenv().unwrap();
        let store = BatteryStore::new(10, Some(env::var("DATABASE_URL").unwrap()))
            .await
            .unwrap();
        store
    }
    #[tokio::test]
    async fn down_sample() {
        let mut store = get_store().await;
        let mut battery = battery::Status::build().unwrap()[0].clone();
        for _ in 0..1500 {
            battery.last();
            let system = system::Status::build().unwrap();
            let row = store.insert(&battery, &system).await;
            //在系统进入休眠或睡眠瞬间，insert会Err，需要处理
            match row {
                Ok(inner) => {
                    inner.map(|x| {
                        assert_eq!(x.timestamp, battery.timestamp);
                    });
                }
                Err(e) => {
                    dbg!(e);
                }
            }
            tokio::time::sleep(Duration::from_secs_f32(0.001)).await;
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
