//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.4

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "battery_state_history")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub timestamp: i64,
    pub state: String,
    pub prev: Option<String>,
    pub end_at: Option<i64>,
    #[sea_orm(column_type = "Float")]
    pub capacity: f32,
    #[sea_orm(column_type = "Float")]
    pub full_capacity: f32,
    #[sea_orm(column_type = "Float")]
    pub design_capacity: f32,
    #[sea_orm(column_type = "Float")]
    pub percentage: f32,
    #[sea_orm(column_type = "Float")]
    pub state_of_health: f32,
    #[sea_orm(column_type = "Float")]
    pub energy_rate: f32,
    #[sea_orm(column_type = "Float")]
    pub voltage: f32,
    #[sea_orm(column_type = "Float")]
    pub cpu_load: f32,
    #[sea_orm(column_type = "Float")]
    pub screen_brightness: f32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
