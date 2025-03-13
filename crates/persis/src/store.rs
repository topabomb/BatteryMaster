use crate::battery_one_minutes;
use crate::battery_realtime;
use crate::battery_state_history;
use crate::down_sample::*;
use crate::memory_battery_status;
use chrono::prelude::*;
use chrono::{DateTime, Duration, Utc};
use migration::*;
use sea_orm::*;
pub struct BatteryStore {
    pub db: Option<DatabaseConnection>,
    pub mem_db: DatabaseConnection,
    last_save_at: i64,
    interval_secs: u32,
    history_need_init: bool,
}
impl Default for BatteryStore {
    fn default() -> Self {
        Self {
            mem_db: DatabaseConnection::Disconnected,
            db: None,
            last_save_at: 0,
            interval_secs: 10,
            history_need_init: false,
        }
    }
}
impl BatteryStore {
    pub async fn new(interval_secs: u32, conn_str: Option<String>) -> Result<Self, DbErr> {
        let db_url = match conn_str {
            Some(val) => val,
            None => String::from("sqlite::memory:"),
        };
        let db = Database::connect(db_url).await?;
        let mem_db = Database::connect(String::from("sqlite::memory:")).await?;
        let db_is_new = !(SchemaManager::new(&db)
            .has_table("battery_realtime")
            .await?);
        migration::Migrator::up(&db, None).await?;
        migration::Migrator::up(&mem_db, None).await?;
        let instance = Self {
            last_save_at: Utc::now().timestamp(),
            interval_secs,
            db: Some(db),
            mem_db,
            history_need_init: db_is_new,
        };
        Ok(instance)
    }
    pub async fn insert(
        &mut self,
        battery: &battery::Status,
        system: &system::Status,
    ) -> Result<Option<memory_battery_status::Model>, DbErr> {
        let status = memory_battery_status::ActiveModel {
            timestamp: ActiveValue::Set(battery.timestamp),
            state: ActiveValue::Set(battery.state.to_string()),
            percentage: ActiveValue::Set(battery.percentage),
            state_of_health: ActiveValue::Set(battery.state_of_health),
            energy_rate: ActiveValue::Set(battery.energy_rate),
            voltage: ActiveValue::Set(battery.voltage),
            cpu_load: ActiveValue::Set(system.cpuload),
            screen_brightness: ActiveValue::Set(system.screen_brightness),
            ..Default::default()
        };
        let model = status.insert(&self.mem_db).await;
        match model {
            Err(err) => {
                println!("insert to memory_battery_status error:%{}", err);
                if err
                    .to_string()
                    .contains("no such table: memory_battery_status")
                {
                    //时间长了会出现这个错误，不理解为什么，故丢弃数据，重置这个数据库连接
                    self.mem_db = Database::connect(String::from("sqlite::memory:")).await?;
                    Migrator::up(&self.mem_db, None).await?;
                    Ok(None)
                } else {
                    Err(err)
                }
            }
            Ok(model) => {
                let now = Utc::now().timestamp();
                self.history(&now, &battery, &system).await?;
                if (self.last_save_at + self.interval_secs as i64) < now {
                    self.last_save_at = now;
                    self.clean(&now).await?;
                    self.merge(&now).await?;
                    self.update_history_avg(&now, None).await?;
                }
                Ok(Some(model))
            }
        }
    }
    async fn update_history_avg(
        &mut self,
        now: &i64,
        history: Option<battery_state_history::Model>,
    ) -> Result<(), DbErr> {
        let db = self.db.as_ref().unwrap();
        let model = match history {
            Some(v) => v,
            None => {
                let last = battery_state_history::Entity::find()
                    .filter(battery_state_history::Column::Timestamp.lt(*now))
                    .order_by_desc(battery_state_history::Column::Timestamp)
                    .one(db)
                    .await?;
                if last.is_none() {
                    return Ok(());
                }
                last.unwrap()
            }
        };
        if model.end_at.is_none() {
            let func_str = match model.state.as_str() {
                "full" => "MAX",
                "charging" => "MAX",
                "discharging" => "MIN",
                _ => "AVG",
            };
            let sql = format!(
                r#"
SELECT 
    {func_str}(percentage) as percentage,
    MIN(state_of_health) as state_of_health,
    AVG(energy_rate) as energy_rate,
    AVG(voltage) as voltage,
    AVG(cpu_load) as cpu_load,
    AVG(screen_brightness) as screen_brightness
FROM battery_one_minutes
WHERE timestamp BETWEEN {} AND {}
        "#,
                model.timestamp, now
            );
            let rows = db
                .query_one(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    sql,
                ))
                .await?;
            let mut model = model.into_active_model();
            if let Some(rows) = rows {
                let res = rows.try_get_many::<(f32, f32, f32, f32, f32, f32)>(
                    "",
                    &[
                        "percentage".to_string(),
                        "state_of_health".to_string(),
                        "energy_rate".to_string(),
                        "voltage".to_string(),
                        "cpu_load".to_string(),
                        "screen_brightness".to_string(),
                    ],
                );
                if res.is_ok() {
                    res.map(
                        |(
                            percentage,
                            state_of_health,
                            energy_rate,
                            voltage,
                            cpu_load,
                            screen_brightness,
                        )| {
                            model.percentage = Set(percentage);
                            model.state_of_health = Set(state_of_health);
                            model.energy_rate = Set(energy_rate);
                            model.voltage = Set(voltage);
                            model.cpu_load = Set(cpu_load);
                            model.screen_brightness = Set(screen_brightness);
                        },
                    )?;
                    model = model.save(db).await?;
                    println!(
                        "update_history_avg update for timestamp({}).",
                        model.timestamp.unwrap()
                    );
                }
            }
        }
        Ok(())
    }
    async fn history(
        &mut self,
        now: &i64,
        battery: &battery::Status,
        system: &system::Status,
    ) -> Result<(), DbErr> {
        let db = &self.db.as_ref().unwrap().clone();
        if !battery.state_changed {
            if self.history_need_init {
                self.history_need_init = false;
                if battery_state_history::Entity::find()
                    .one(db)
                    .await?
                    .is_none()
                {
                    let res =
                        battery_state_history::Entity::insert(battery_state_history::ActiveModel {
                            timestamp: Set(*now),
                            state: Set(battery.state.to_string()),
                            capacity: Set(battery.capacity),
                            full_capacity: Set(battery.full_capacity),
                            design_capacity: Set(battery.design_capacity),
                            percentage: Set(battery.percentage),
                            state_of_health: Set(battery.state_of_health),
                            energy_rate: Set(battery.energy_rate),
                            voltage: Set(battery.voltage),
                            cpu_load: Set(system.cpuload),
                            screen_brightness: Set(system.screen_brightness),
                            prev: Set(None),
                            end_at: Set(None),
                        })
                        .exec(db)
                        .await?;
                    println!("history init of last_insert_id({})", res.last_insert_id);
                }
            }
            return Ok(());
        }
        let prev = battery_state_history::Entity::find()
            .filter(battery_state_history::Column::Timestamp.lt(*now))
            .order_by_desc(battery_state_history::Column::Timestamp)
            .one(db)
            .await?;
        let model = battery_state_history::ActiveModel {
            timestamp: Set(*now),
            state: Set(battery.state.to_string()),
            prev: Set(prev.as_ref().map(|v| v.state.clone())),
            end_at: Set(None),
            capacity: Set(battery.capacity),
            full_capacity: Set(battery.full_capacity),
            design_capacity: Set(battery.design_capacity),
            percentage: Set(battery.percentage),
            state_of_health: Set(battery.state_of_health),
            energy_rate: Set(battery.energy_rate),
            voltage: Set(battery.voltage),
            cpu_load: Set(system.cpuload),
            screen_brightness: Set(system.screen_brightness),
        };
        if let Some(prev) = prev {
            let mut prev_model = prev.clone().into_active_model();
            self.update_history_avg(now, Some(prev)).await?;
            prev_model.end_at = Set(Some(*now));
            prev_model.save(db).await?;
        }
        let old = battery_state_history::Entity::find_by_id(*now)
            .one(db)
            .await?;
        if old.is_none() {
            let res = battery_state_history::Entity::insert(model)
                .exec(db)
                .await?;
            println!("history insert of last_insert_id({})", res.last_insert_id);
        } else {
            let res = battery_state_history::Entity::update(model)
                .exec(db)
                .await?;
            println!("history update of timestamp({})", res.timestamp);
        }
        Ok(())
    }
    async fn clean(&mut self, now: &i64) -> Result<(), DbErr> {
        let db = self.db.as_ref().unwrap();
        //clean memory_battery_status
        let clean_point = now - Duration::minutes(1).num_seconds();
        let res = memory_battery_status::Entity::delete_many()
            .filter(memory_battery_status::Column::Timestamp.lt(clean_point))
            .exec(&self.mem_db)
            .await?;
        println!(
            "memory_battery_status delete of rows_affected({})",
            res.rows_affected
        );
        //clean battery_realtime
        let clean_point = now - Duration::hours(6).num_seconds();
        let res = battery_realtime::Entity::delete_many()
            .filter(battery_realtime::Column::Timestamp.lt(clean_point))
            .exec(db)
            .await?;
        println!(
            "battery_realtime delete of rows_affected({})",
            res.rows_affected
        );
        //clean battery_one_minutes
        let clean_point = now - Duration::days(30).num_seconds();
        let res = battery_one_minutes::Entity::delete_many()
            .filter(battery_one_minutes::Column::Timestamp.lt(clean_point))
            .exec(db)
            .await?;
        println!(
            "battery_one_minutes delete of rows_affected({})",
            res.rows_affected
        );
        Ok(())
    }
    async fn merge(&mut self, now: &i64) -> Result<(), DbErr> {
        let db = self.db.as_ref().unwrap();
        //memory_battery_status to battery_realtime
        let insert_rows: Vec<battery_realtime::ActiveModel> = down_sample(
            &self.mem_db,
            &DownSampleParams {
                table_name: "memory_battery_status".to_string(),
                end_time: *now,
                start_time: now - self.interval_secs as i64,
                interval_secs: 1,
                order_field: "id".to_string(),
                ..Default::default()
            },
        )
        .await?
        .iter()
        .map(|x| {
            battery_realtime::Model {
                timestamp: x.timestamp,
                percentage: x.percentage,
                state: x.state.clone(),
                energy_rate: x.energy_rate,
                voltage: x.voltage,
                cpu_load: x.cpu_load,
                screen_brightness: x.screen_brightness,
                state_of_health: x.state_of_health,
            }
            .into_active_model()
        })
        .collect();
        let res = battery_realtime::Entity::insert_many(insert_rows)
            .exec(db)
            .await?;
        println!(
            "memory_battery_status to battery_realtime of last_insert_id({})",
            res.last_insert_id
        );
        //battery_realtime to battery_one_minutes
        let now_instance = DateTime::<Utc>::from_timestamp(*now, 0).unwrap();
        let current_minute = now_instance.trunc_subsecs(0).with_second(0).unwrap();
        let previous_minute = current_minute - Duration::minutes(1);
        let update_rows: Vec<battery_one_minutes::ActiveModel> = down_sample(
            db,
            &DownSampleParams {
                time_formater: "%Y-%m-%dT%H:%M:00Z".to_string(),
                table_name: "battery_realtime".to_string(),
                end_time: *now,
                start_time: previous_minute.timestamp(),
                interval_secs: 1,
                order_field: "timestamp".to_string(),
                ..Default::default()
            },
        )
        .await?
        .iter()
        .map(|x| {
            battery_one_minutes::Model {
                timestamp: x.timestamp,
                percentage: x.percentage,
                state: x.state.clone(),
                energy_rate: x.energy_rate,
                voltage: x.voltage,
                cpu_load: x.cpu_load,
                screen_brightness: x.screen_brightness,
                state_of_health: x.state_of_health,
            }
            .into_active_model()
        })
        .collect();
        for mut row in update_rows {
            let old = battery_one_minutes::Entity::find_by_id(row.timestamp.clone().unwrap())
                .one(db)
                .await?;
            if old.is_none() {
                let res = battery_one_minutes::Entity::insert(row).exec(db).await?;
                println!(
                    "battery_realtime to battery_one_minutes insert of timestamp({})",
                    res.last_insert_id
                );
            } else {
                let res = battery_one_minutes::Entity::update(row).exec(db).await?;
                println!(
                    "battery_realtime to battery_one_minutes update of timestamp({})",
                    res.timestamp
                );
            }
        }
        Ok(())
    }
}
