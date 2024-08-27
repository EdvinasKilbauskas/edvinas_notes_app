use axum::http::{ HeaderName, HeaderValue };
use insta::{ assert_debug_snapshot, with_settings };
use loco_rs::testing;
use edvinas_notes_app::{ app::App, models::_entities::notes::Entity };
use edvinas_notes_app::models::_entities::note_shares;
use edvinas_notes_app::models::_entities::notes;
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serial_test::serial;

// TODO: see how to dedup / extract this to app-local test utils
// not to framework, because that would require a runtime dep on insta
macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("notes_request");
        let _guard = settings.bind_to_scope();
    };
}

async fn authenticate_user(
    mut request: loco_rs::TestServer,
    email: &str,
    password: &str
) -> loco_rs::TestServer {
    let login_payload =
        serde_json::json!({
        "email": email,
        "password": password,
    });

    let login_response = request.post("/api/auth/login").json(&login_payload).await;

    assert_eq!(login_response.status_code(), 200);

    let body: serde_json::Value = serde_json::from_str(&login_response.text()).unwrap();
    let token = body["token"].as_str().unwrap();

    request.add_header(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap()
    );
    request
}

#[tokio::test]
#[serial]
async fn can_get_notes() {
    configure_insta!();

    testing::request::<App, _, _>(|request: loco_rs::TestServer, ctx| async move {
        testing::seed::<App>(&ctx.db).await.unwrap();
        let authenticated_request = authenticate_user(request, "edvinas1@gmail.com", "1234").await;

        let notes = authenticated_request.get("/api/notes").await;

        with_settings!({
            filters => {
                 let mut combined_filters = testing::CLEANUP_DATE.to_vec();
                    combined_filters.extend(vec![(r#"\"id\\":\d+"#, r#""id\":ID"#)]);
                    combined_filters
            }
        }, {
            assert_debug_snapshot!(
            (notes.status_code(), notes.text())
        );
        });
    }).await;
}

#[tokio::test]
#[serial]
async fn can_add_note() {
    configure_insta!();

    testing::request::<App, _, _>(|request, ctx| async move {
        testing::seed::<App>(&ctx.db).await.unwrap();

        let authenticated_request = authenticate_user(request, "edvinas1@gmail.com", "1234").await;

        let payload =
            serde_json::json!({
            "title": "loco",
            "content": "loco note test",
        });

        let add_note_request = authenticated_request.post("/api/notes").json(&payload).await;

        with_settings!({
            filters => {
                 let mut combined_filters = testing::CLEANUP_DATE.to_vec();
                    combined_filters.extend(vec![(r#"\"id\\":\d+"#, r#""id\":ID"#)]);
                    combined_filters
            }
        }, {
            assert_debug_snapshot!(
            (add_note_request.status_code(), add_note_request.text())
        );
        });
    }).await;
}

#[tokio::test]
#[serial]
async fn can_get_note() {
    configure_insta!();

    testing::request::<App, _, _>(|request, ctx| async move {
        testing::seed::<App>(&ctx.db).await.unwrap();

        let authenticated_request = authenticate_user(request, "edvinas1@gmail.com", "1234").await;

        let get_note_request = authenticated_request.get("/api/notes/3").await;

        with_settings!({
            filters => {
                 let mut combined_filters = testing::CLEANUP_DATE.to_vec();
                    combined_filters.extend(vec![(r#"\"id\\":\d+"#, r#""id\":ID"#)]);
                    combined_filters
            }
        }, {
            assert_debug_snapshot!(
            (get_note_request.status_code(), get_note_request.text())
        );
        });
    }).await;
}

#[tokio::test]
#[serial]
async fn can_delete_note() {
    configure_insta!();

    testing::request::<App, _, _>(|request, ctx| async move {
        testing::seed::<App>(&ctx.db).await.unwrap();

        let authenticated_request = authenticate_user(request, "edvinas1@gmail.com", "1234").await;
        let count_before_delete = Entity::find().all(&ctx.db).await.unwrap().len();
        let delete_note_request = authenticated_request.delete("/api/notes/3").await;

        with_settings!({
            filters => {
                 let mut combined_filters = testing::CLEANUP_DATE.to_vec();
                    combined_filters.extend(vec![(r#"\"id\\":\d+"#, r#""id\":ID"#)]);
                    combined_filters
            }
        }, {
            assert_debug_snapshot!(
            (delete_note_request.status_code(), delete_note_request.text())
        );
        });

        let count_after_delete = Entity::find().all(&ctx.db).await.unwrap().len();
        assert_eq!(count_after_delete, count_before_delete - 1);
    }).await;
}

#[tokio::test]
#[serial]
async fn can_share_note() {
    configure_insta!();

    testing::request::<App, _, _>(|request, ctx| async move {
        testing::seed::<App>(&ctx.db).await.unwrap();
        let authenticated_request = authenticate_user(request, "edvinas1@gmail.com", "1234").await;
        // find note by user
        let note_id = 3;

        let payload = serde_json::json!({
            "shared_with_user_id": 4
        });

        let share_note_request = authenticated_request
            .post(&format!("/api/notes/{}/share", note_id))
            .json(&payload).await;

        with_settings!({
            filters => {
                 let mut combined_filters = testing::CLEANUP_DATE.to_vec();
                 combined_filters.extend(vec![(r#"\"id\\":\d+"#, r#""id\":ID"#)]);
                 combined_filters
            }
        }, {
            assert_debug_snapshot!(
                (share_note_request.status_code(), share_note_request.text())
            );
        });

        // Verify the share was created
        let share = note_shares::Entity
            ::find()
            .filter(note_shares::Column::NoteId.eq(note_id))
            .one(&ctx.db).await
            .unwrap();
        assert!(share.is_some());
    }).await;
}

#[tokio::test]
#[serial]
async fn can_share_all_notes() {
    configure_insta!();

    testing::request::<App, _, _>(|request, ctx| async move {
        testing::seed::<App>(&ctx.db).await.unwrap();

        let authenticated_request = authenticate_user(request, "edvinas1@gmail.com", "1234").await;

        let payload = serde_json::json!({
            "shared_with_user_id": 4
        });

        // delete all notes shared with user 4
        note_shares::Entity
            ::delete_many()
            .filter(note_shares::Column::SharedWithUserId.eq(4))
            .exec(&ctx.db).await
            .unwrap();

        let share_all_notes_request = authenticated_request
            .post("/api/notes/share-all")
            .json(&payload).await;

        with_settings!(
            {},
            {
                assert_debug_snapshot!((
                    share_all_notes_request.status_code(),
                    share_all_notes_request.text(),
                ));
            }
        );

        // Verify that all notes are shared
        let shared_notes = note_shares::Entity
            ::find()
            .filter(note_shares::Column::SharedWithUserId.eq(4))
            .all(&ctx.db).await
            .unwrap();

        let user_notes = notes::Entity
            ::find()
            .filter(notes::Column::UserId.eq(3))
            .all(&ctx.db).await
            .unwrap();

        assert_eq!(shared_notes.len(), user_notes.len());
    }).await;
}

#[tokio::test]
#[serial]
async fn can_get_shared_notes() {
    configure_insta!();

    testing::request::<App, _, _>(|request, ctx| async move {
        testing::seed::<App>(&ctx.db).await.unwrap();
        let authenticated_request = authenticate_user(request, "edvinas1@gmail.com", "1234").await;
        let note = Entity::find().one(&ctx.db).await.unwrap().unwrap();

        let _share = (note_shares::ActiveModel {
            note_id: Set(note.id),
            shared_with_user_id: Set(2),
            ..Default::default()
        })
            .insert(&ctx.db).await
            .unwrap();

        let get_shared_notes_request = authenticated_request.get("/api/notes/shared").await;

        with_settings!({
            filters => {
                 let mut combined_filters = testing::CLEANUP_DATE.to_vec();
                 combined_filters.extend(vec![(r#"\"id\\":\d+"#, r#""id\":ID"#)]);
                 combined_filters
            }
        }, {
            assert_debug_snapshot!(
                (get_shared_notes_request.status_code(), get_shared_notes_request.text())
            );
        });
    }).await;
}

#[tokio::test]
#[serial]
async fn can_update_shared_note() {
    configure_insta!();

    testing::request::<App, _, _>(|request, ctx| async move {
        testing::seed::<App>(&ctx.db).await.unwrap();
        let authenticated_request = authenticate_user(request, "edvinas1@gmail.com", "1234").await;
        // Assume user 1 and 2 exist from seeding
        let note = Entity::find().one(&ctx.db).await.unwrap().unwrap();

        // Share the note
        let _share = (note_shares::ActiveModel {
            note_id: Set(note.id),
            shared_with_user_id: Set(4), // Assume user 4
            ..Default::default()
        })
            .insert(&ctx.db).await
            .unwrap();

        // Update the note
        let payload =
            serde_json::json!({
            "title": "Updated Shared Note",
            "content": "This note has been updated",
        });

        let update_note_request = authenticated_request
            .post(&format!("/api/notes/{}", note.id))
            .json(&payload).await;

        with_settings!({
            filters => {
                 let mut combined_filters = testing::CLEANUP_DATE.to_vec();
                 combined_filters.extend(vec![(r#"\"id\\":\d+"#, r#""id\":ID"#)]);
                 combined_filters
            }
        }, {
            assert_debug_snapshot!(
                (update_note_request.status_code(), update_note_request.text())
            );
        });

        // Verify the update
        let updated_note = Entity::find_by_id(note.id).one(&ctx.db).await.unwrap().unwrap();
        assert_eq!(updated_note.title, Some("Updated Shared Note".to_string()));
    }).await;
}
