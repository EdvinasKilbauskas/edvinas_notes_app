use sea_orm::entity::prelude::*;
use serde::{ Deserialize, Serialize };

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "note_shares")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub note_id: i32,
    pub shared_with_user_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::notes::Entity",
        from = "Column::NoteId",
        to = "super::notes::Column::Id"
    )]
    Note,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::SharedWithUserId",
        to = "super::users::Column::Id"
    )]
    SharedWithUser,
}

impl Related<super::notes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Note.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SharedWithUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
