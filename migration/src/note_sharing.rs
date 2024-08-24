use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(NoteAccess::Table)
                .if_not_exists()
                .col(ColumnDef::new(NoteAccess::Id).integer().not_null().primary_key().auto_increment())
                .col(ColumnDef::new(NoteAccess::NoteId).integer().not_null())
                .col(ColumnDef::new(NoteAccess::OwnerId).integer().not_null())
                .col(ColumnDef::new(NoteAccess::UserId).integer().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-note_access-note_id")
                        .from(NoteAccess::Table, NoteAccess::NoteId)
                        .to(Notes::Table, Notes::Id), // Correctly references Notes::Id
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-note_access-user_id")
                        .from(NoteAccess::Table, NoteAccess::UserId)
                        .to(Users::Table, Users::Id), // Correctly references Users::Id
                )
                .to_owned(),
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(NoteAccess::Table).to_owned()).await?;

        Ok(())
    }
}

#[derive(Iden)]
enum NoteAccess {
    Table,
    Id,
    NoteId,
    OwnerId,
    UserId,
}

#[derive(Iden)]
enum Notes {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}