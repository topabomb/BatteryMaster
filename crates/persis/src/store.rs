use crate::battery_one_minutes;
use crate::battery_realtime;
use crate::battery_state_history;
use crate::down_sample::*;
use crate::memory_battery_status;
use chrono::prelude::*;
use chrono::{DateTime, Duration, Utc};
use migration::*;
use sea_orm::*;
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InsertModifyed {
    Unknown,
    BatteryHistory,
}
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
    pub async fn insert<F>(
        &mut self,
        battery: &battery::Status,
        system: &system::Status,
        f: F,
    ) -> Result<(Vec<InsertModifyed>, Option<memory_battery_status::Model>), DbErr>
    where
        F: AsyncFnOnce(i64) -> (),
    {
        let status = memory_battery_status::ActiveModel {
            timestamp: ActiveValue::Set(battery.timestamp),
            state: ActiveValue::Set(battery.state.to_string()),
            percentage: ActiveValue::Set(battery.percentage),
            energy_rate: ActiveValue::Set(battery.energy_rate),
            voltage: ActiveValue::Set(battery.voltage),
            cpu_load: ActiveValue::Set(system.cpuload),
            ..Default::default()
        };
        let mut changed_vec: Vec<InsertModifyed> = Vec::new();
        let mut model = status.clone().insert(&self.mem_db).await;
        if model.is_err() {
            let err = model.err().unwrap();
            println!("insert to memory_battery_status error:%{}", err);
            if err
                .to_string()
                .contains("no such table: memory_battery_status")
            {
                //时间长了会出现这个错误，不理解为什么，故丢弃数据，重置这个数据库连接
                self.mem_db = Database::connect(String::from("sqlite::memory:")).await?;
                Migrator::up(&self.mem_db, None).await?;
                model = status.insert(&self.mem_db).await;
            } else {
                return Err(err);
            }
        }
        match model {
            Ok(model) => {
                let now = battery.timestamp;
                let new_history = self.history(&now, &battery, &system).await?;
                let mut result_history = None;
                if (self.last_save_at + self.interval_secs as i64) < now {
                    self.last_save_at = now;
                    self.clean(&now).await?;
                    self.merge(&now).await?;
                    result_history = self.update_last_history(&now, battery, system).await?;
                }
                if result_history.is_some() || new_history.is_some() {
                    changed_vec.push(InsertModifyed::BatteryHistory);
                }
                match result_history {
                    Some(h) => f(h).await,
                    None => match new_history {
                        Some(h) => f(h).await,
                        None => (),
                    },
                }
                Ok((changed_vec, Some(model)))
            }
            Err(e) => Err(e),
        }
    }

    async fn history(
        &mut self,
        now: &i64,
        battery: &battery::Status,
        system: &system::Status,
    ) -> Result<Option<i64>, DbErr> {
        let db = &self.db.as_ref().unwrap().clone();
        let mut last_insert_id = None;
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
                            prev: Set(None),
                            end_at: Set(None),
                        })
                        .exec(db)
                        .await?;
                    println!("history init of last_insert_id({})", res.last_insert_id);
                    last_insert_id = Some(res.last_insert_id)
                }
            }
            return Ok(last_insert_id);
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
        };
        if let Some(prev) = prev {
            let mut prev_model = prev.clone().into_active_model();
            self.update_history_avg(now, prev, battery, system, Some(*now))
                .await?;
            prev_model.state_of_health = Set(battery.state_of_health);
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
            last_insert_id = Some(res.last_insert_id);
        } else {
            let res = battery_state_history::Entity::update(model)
                .exec(db)
                .await?;
            println!("history update of timestamp({})", res.timestamp);
        }
        Ok(last_insert_id)
    }
    async fn update_last_history(
        &mut self,
        now: &i64,
        battery: &battery::Status,
        system: &system::Status,
    ) -> Result<Option<i64>, DbErr> {
        let db = self.db.as_ref().unwrap();
        let last = battery_state_history::Entity::find()
            .filter(battery_state_history::Column::Timestamp.lt(*now))
            .order_by_desc(battery_state_history::Column::Timestamp)
            .one(db)
            .await?;
        let new_model = battery_state_history::ActiveModel {
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
            prev: Set(None),
            end_at: Set(None),
        };
        let new_id = match &last {
            None => {
                let res =
                    battery_state_history::Entity::insert(battery_state_history::ActiveModel {
                        prev: Set(None),
                        end_at: Set(None),
                        ..new_model
                    })
                    .exec(db)
                    .await?;
                Some(res.last_insert_id)
            }
            Some(last) => {
                if last.end_at.is_none()
                    && last.state != battery.state.to_string()
                    && battery.state != battery::State(battery::ExternalBatteryState::Unknown)
                {
                    let res =
                        battery_state_history::Entity::insert(battery_state_history::ActiveModel {
                            prev: Set(Some(last.state.clone())),
                            end_at: Set(None),
                            ..new_model
                        })
                        .exec(db)
                        .await?;

                    Some(res.last_insert_id)
                } else {
                    None
                }
            }
        };
        if last.is_some() {
            let last = last.unwrap();
            self.update_history_avg(
                now,
                last,
                battery,
                system,
                match new_id.is_some() {
                    true => Some(*now),
                    false => None,
                },
            )
            .await?;
        }
        Ok(new_id)
    }
    async fn update_history_avg(
        &mut self,
        now: &i64,
        history: battery_state_history::Model,
        battery: &battery::Status,
        system: &system::Status,
        end: Option<i64>,
    ) -> Result<(), DbErr> {
        let model = history;
        let db = self.db.as_ref().unwrap();
        if model.end_at.is_none() {
            let min_start_at = *now - Duration::days(7).num_seconds();
            let timestamp = match model.timestamp < min_start_at {
                true => min_start_at,
                false => model.timestamp,
            };
            let sql = format!(
                r#"
SELECT 
    AVG(energy_rate) as energy_rate,
    AVG(voltage) as voltage,
    AVG(cpu_load) as cpu_load
FROM battery_one_minutes
WHERE timestamp BETWEEN {timestamp} AND {now}
"#
            );
            let rows = db
                .query_one(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    sql,
                ))
                .await?;
            let mut model = model.into_active_model();
            if let Some(rows) = rows {
                let res = rows.try_get_many::<(f32, f32, f32)>(
                    "",
                    &[
                        "energy_rate".to_string(),
                        "voltage".to_string(),
                        "cpu_load".to_string(),
                    ],
                );
                if res.is_ok() {
                    res.map(|(energy_rate, voltage, cpu_load)| {
                        model.energy_rate = Set(energy_rate);
                        model.voltage = Set(voltage);
                        model.cpu_load = Set(cpu_load);
                    })?;
                }
                model.percentage = Set(battery.percentage);
                model.state_of_health = Set(battery.state_of_health);
                model.capacity = Set(battery.capacity);
                model.full_capacity = Set(battery.full_capacity);
                model.design_capacity = Set(battery.design_capacity);
                end.map(|end| {
                    model.end_at = Set(Some(end));
                });
                model = model.save(db).await?;
                println!(
                    "update_history_avg update for timestamp({}).",
                    model.timestamp.unwrap()
                );
            }
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
        let clean_point = now - Duration::days(1).num_seconds();
        let res = battery_realtime::Entity::delete_many()
            .filter(battery_realtime::Column::Timestamp.lt(clean_point))
            .exec(db)
            .await?;
        println!(
            "battery_realtime delete of rows_affected({})",
            res.rows_affected
        );
        //clean battery_one_minutes
        let clean_point = now - Duration::days(7).num_seconds();
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
                interval_secs: 2,
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
