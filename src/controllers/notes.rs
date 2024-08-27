#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::debug_handler;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use crate::models::_entities::notes::{ActiveModel, Column, Entity, Model};
use crate::models::_entities::users;
use crate::models::_entities::note_shares::{self, ActiveModel as NoteShareActiveModel};
use sea_orm::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareNoteParams {
    pub shared_with_user_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedNoteResponse {
    pub id: i32,
    pub title: Option<String>,
    pub content: Option<String>,
    pub shared_by_user_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub title: Option<String>,
    pub content: Option<String>,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.title = Set(self.title.clone());
        item.content = Set(self.content.clone());
    }
}

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};

async fn load_item(ctx: &AppContext, id: i32, user_id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id)
        .filter(
            crate::models::_entities::notes::Column::UserId.eq(user_id)
                .or(
                    crate::models::_entities::note_shares::Column::NoteId.eq(id)
                        .and(crate::models::_entities::note_shares::Column::SharedWithUserId.eq(user_id))
                )
        )
        .join(
            JoinType::LeftJoin,
            crate::models::_entities::notes::Relation::NoteShares.def()
        )
        .one(&ctx.db)
        .await?;
    
    item.ok_or_else(|| Error::NotFound)
}


#[debug_handler]
pub async fn list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    
    let notes = Entity::find()
        .filter(
            Condition::any()
                .add(crate::models::_entities::notes::Column::UserId.eq(user.id))
                .add(
                    crate::models::_entities::notes::Column::Id.in_subquery(
                        note_shares::Entity::find()
                            .select_only()
                            .column(note_shares::Column::NoteId)
                            .filter(note_shares::Column::SharedWithUserId.eq(user.id))
                            .into_query()
                    )
                )
        )
        .all(&ctx.db)
        .await?;
    
    format::json(notes)
}

#[debug_handler]
pub async fn add(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<Params>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let mut item = ActiveModel {
        user_id: Set(user.id),
        
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn update(
    auth: auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = Entity::find_by_id(id)
        .filter(crate::models::_entities::notes::Column::UserId.eq(user.id))
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn remove(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = Entity::find_by_id(id)
        .filter(crate::models::_entities::notes::Column::UserId.eq(user.id))
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    item.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_notes_shared_by_me(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    
    let shared_notes = Entity::find()
        .filter(crate::models::_entities::notes::Column::UserId.eq(user.id))
        .join(JoinType::InnerJoin, crate::models::_entities::notes::Relation::NoteShares.def())
        .group_by(crate::models::_entities::notes::Column::Id)
        .all(&ctx.db)
        .await?;
    
    format::json(shared_notes)
}

#[debug_handler]
pub async fn get_one(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    format::json(load_item(&ctx, id, user.id).await?)
}

#[debug_handler]
pub async fn share_note(
    auth: auth::JWT,
    Path(note_id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<ShareNoteParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let note = load_item(&ctx, note_id, user.id).await?;
    
    let share = NoteShareActiveModel {
        note_id: Set(note.id),
        shared_with_user_id: Set(params.shared_with_user_id),
        ..Default::default()
    };
    let share = share.insert(&ctx.db).await?;
    
    format::json(share)
}



#[debug_handler]
pub async fn share_all_notes(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<ShareNoteParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    
    // Get all notes of the current user
    let user_notes = Entity::find()
        .filter(Column::UserId.eq(user.id))
        .all(&ctx.db)
        .await?;

    // Share each note with the specified user
    for note in user_notes {
        let share = note_shares::ActiveModel {
            note_id: Set(note.id),
            shared_with_user_id: Set(params.shared_with_user_id),
            ..Default::default()
        };
        share.insert(&ctx.db).await?;
    }

    format::json(serde_json::json!({
        "message": "All notes have been shared successfully"
    }))
}

#[debug_handler]
pub async fn get_shared_notes(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    
    let shared_notes = Entity::find()
        .join(JoinType::InnerJoin, crate::models::_entities::notes::Relation::NoteShares.def())
        .filter(crate::models::_entities::note_shares::Column::SharedWithUserId.eq(user.id))
        .group_by(crate::models::_entities::notes::Column::Id)
        .all(&ctx.db)
        .await?;
    
    format::json(shared_notes)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("notes")
        .add("/", get(list))
        .add("/", post(add))
        .add("/:id", get(get_one))
        .add("/:id", delete(remove))
        .add("/:id", post(update))
        .add("/:id/share", post(share_note))
        .add("/shared", get(get_shared_notes))
        .add("/shared-by-me", get(get_notes_shared_by_me))
        .add("/share-all", post(share_all_notes))
}