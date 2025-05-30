use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MemoryBatteryStatus::Table)
                    .if_not_exists()
                    .col(pk_auto(MemoryBatteryStatus::Id))
                    .col(big_integer(MemoryBatteryStatus::Timestamp))
                    .col(string(MemoryBatteryStatus::State))
                    .col(float(MemoryBatteryStatus::Percentage))
                    .col(float(MemoryBatteryStatus::EnergyRate))
                    .col(float(MemoryBatteryStatus::Voltage))
                    .col(float(MemoryBatteryStatus::CpuLoad))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(MemoryBatteryStatus::Table)
                    .name("idx_timestamp")
                    .col(MemoryBatteryStatus::Timestamp)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(MemoryBatteryStatus::Table)
                    .name("idx_MemoryBatteryStatus_state")
                    .col(MemoryBatteryStatus::State)
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(BatteryStateHistory::Table)
                    .if_not_exists()
                    .col(big_integer(BatteryStateHistory::Timestamp).primary_key())
                    .col(string(BatteryStateHistory::State))
                    .col(ColumnDef::new(BatteryStateHistory::Prev).string().null())//这么写才能定义null
                    .col(ColumnDef::new(BatteryStateHistory::EndAt).big_integer().null())
                    .col(float(BatteryStateHistory::Capacity))
                    .col(float(BatteryStateHistory::FullCapacity))
                    .col(float(BatteryStateHistory::DesignCapacity))
                    .col(float(BatteryStateHistory::Percentage))
                    .col(float(BatteryStateHistory::StateOfHealth))
                    .col(float(BatteryStateHistory::EnergyRate))
                    .col(float(BatteryStateHistory::Voltage))
                    .col(float(BatteryStateHistory::CpuLoad))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(BatteryStateHistory::Table)
                    .name("idx_BatteryStateHistory_state")
                    .col(BatteryStateHistory::State)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(BatteryStateHistory::Table)
                    .name("idx_BatteryStateHistory_prev")
                    .col(BatteryStateHistory::Prev)
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(BatteryRealtime::Table)
                    .if_not_exists()
                    .col(big_integer(BatteryRealtime::Timestamp).primary_key())
                    .col(string(BatteryRealtime::State))
                    .col(float(BatteryRealtime::Percentage))
                    .col(float(BatteryRealtime::EnergyRate))
                    .col(float(BatteryRealtime::Voltage))
                    .col(float(BatteryRealtime::CpuLoad))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(BatteryOneMinutes::Table)
                    .if_not_exists()
                    .col(big_integer(BatteryOneMinutes::Timestamp).primary_key())
                    .col(string(BatteryOneMinutes::State))
                    .col(float(BatteryOneMinutes::Percentage))
                    .col(float(BatteryOneMinutes::EnergyRate))
                    .col(float(BatteryOneMinutes::Voltage))
                    .col(float(BatteryOneMinutes::CpuLoad))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(MemoryBatteryStatus::Table)
                    .name("idx_timestamp")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .table(MemoryBatteryStatus::Table)
                    .name("idx_MemoryBatteryStatus_state")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .table(BatteryStateHistory::Table)
                    .name("idx_BatteryStateHistory_state")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .table(BatteryStateHistory::Table)
                    .name("idx_BatteryStateHistory_prev")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(MemoryBatteryStatus::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BatteryStateHistory::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BatteryRealtime::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BatteryOneMinutes::Table).to_owned())
            .await?;
        Ok(())
    }
}
#[derive(DeriveIden)]
enum MemoryBatteryStatus {
    Table,
    Id,
    Timestamp,
    State,
    Percentage,
    EnergyRate,
    Voltage,
    CpuLoad,
}
#[derive(DeriveIden)]
enum BatteryStateHistory {
    Table,
    Timestamp,
    EndAt,
    State,
    Prev,
    Capacity,
    FullCapacity,
    DesignCapacity,
    Percentage,
    StateOfHealth,
    EnergyRate,
    Voltage,
    CpuLoad,
}
#[derive(DeriveIden)]
enum BatteryRealtime {
    Table,
    Timestamp,
    State,
    Percentage,
    EnergyRate,
    Voltage,
    CpuLoad,
}
#[derive(DeriveIden)]
enum BatteryOneMinutes {
    Table,
    Timestamp,
    State,
    Percentage,
    EnergyRate,
    Voltage,
    CpuLoad,
}
