use std::str::FromStr;

use crate::battery_realtime;
use sea_orm::*;
use chrono::prelude::*;
pub struct DownSampleParams {
    pub time_formater: String,
    pub table_name: String,
    pub time_field: String,
    pub order_field: String,
    pub end_time: i64,
    pub start_time: i64,
    pub interval_secs: i64,
}
impl Default for DownSampleParams {
    fn default() -> Self {
        let end_time = Utc::now().timestamp();
        Self {
            time_formater: "%Y-%m-%dT%H:%M:%SZ".to_string(), //ISO 8601格式
            table_name: "memory_battery_status".to_string(),
            time_field: "timestamp".to_string(),
            order_field: "id".to_string(),
            end_time,
            start_time: end_time - 10,
            interval_secs: 1,
        }
    }
}
pub fn build_down_sample_sql(params: &DownSampleParams) -> String {
    let formater = &params.time_formater;
    let table_name = &params.table_name;
    let time_field = &params.time_field;
    let order_field = &params.order_field;
    let end_time = params.end_time;
    let start_time = params.start_time;
    let interval_secs = params.interval_secs;
    let sql = format!(
        r"
WITH GroupData AS (
SELECT
strftime('{formater}', {time_field}/{interval_secs}*{interval_secs} , 'unixepoch') AS time_group,
AVG(percentage) AS percentage,
AVG(energy_rate) AS energy_rate,
AVG(voltage) AS voltage,
AVG(cpu_load) AS cpu_load,
MAX({time_field}) AS last_timestamp
FROM
{table_name}
WHERE {time_field}>{start_time} and {time_field}<={end_time}
GROUP BY
time_group
),
UniqueBatteryStatus AS (
SELECT
    {time_field},
    state,
    ROW_NUMBER() OVER (PARTITION BY {time_field} ORDER BY {order_field} DESC) AS rn --排序字段用于在分组内的数据排序
FROM
    {table_name}
WHERE {time_field}>{start_time} and {time_field}<={end_time}
)
SELECT
g.time_group,
g.percentage,
g.energy_rate,
g.voltage,
g.cpu_load,
b.state
FROM
GroupData g
JOIN UniqueBatteryStatus b ON
g.last_timestamp = b.{time_field}
AND b.rn = 1
ORDER BY
g.time_group;
    "
    );
    //println!("{}",&sql);
    sql
}
pub async fn down_sample(
    db:&DatabaseConnection,
    params: &DownSampleParams,
) -> Result<Vec<battery_realtime::Model>, DbErr> {
    let res = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        build_down_sample_sql(params),
    ))
    .await?;
    let res: Vec<battery_realtime::Model> = res
        .iter()
        .map(|row| {
            row.try_get_many::<(String, String, f32, f32, f32, f32)>(
                "",
                &[
                    "time_group".to_string(),
                    "state".to_string(),
                    "percentage".to_string(),
                    "energy_rate".to_string(),
                    "voltage".to_string(),
                    "cpu_load".to_string(),
                ],
            )
            .unwrap()
        })
        .map(
            |(
                time_group,
                state,
                percentage,
                energy_rate,
                voltage,
                cpu_load,
            )| {
                battery_realtime::Model {
                    timestamp: DateTime::<Utc>::from_str(time_group.as_str())
                        .unwrap()
                        .timestamp(),
                    state,
                    percentage,
                    energy_rate,
                    voltage,
                    cpu_load,
                }
            },
        )
        .collect();
    Ok(res)
}