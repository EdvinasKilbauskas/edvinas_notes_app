use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NoteShares::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NoteShares::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NoteShares::NoteId).integer().not_null())
                    .col(ColumnDef::new(NoteShares::SharedWithUserId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-note_shares-note_id")
                            .from(NoteShares::Table, NoteShares::NoteId)
                            .to(Notes::Table, Notes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-note_shares-shared_with_user_id")
                            .from(NoteShares::Table, NoteShares::SharedWithUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NoteShares::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum NoteShares {
    Table,
    Id,
    NoteId,
    SharedWithUserId,
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