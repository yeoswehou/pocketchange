use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
enum Message {
    Table,
    Id,
    UserId,
    Content,
    CreatedAt,
    UpdatedAt,
    ParentId,
}


#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(User::Name)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager.create_table(
            Table::create()
                .table(Message::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Message::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(Message::UserId)
                        .integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Message::Content)
                        .text()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Message::ParentId)
                        .integer()
                        .null(),
                )
                .foreign_key(
                    ForeignKeyCreateStatement::new()
                        .name("fk_messages_user_id")
                        .from(Message::Table, Message::UserId)
                        .to(User::Table, User::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKeyCreateStatement::new()
                        .name("fk_messages_parent_id")
                        .from(Message::Table, Message::ParentId)
                        .to(Message::Table, Message::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;
        manager.drop_table(Table::drop().table(Message::Table).to_owned()).await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Post {
    Table,
    Id,
    Title,
    Text,
}
