use crate::graphql::schema::{MutationRoot, MyContext, MySchema, QueryRoot};
use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

async fn graphql_handler(schema: Extension<MySchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    let html = async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    );
    Html(html)
}

pub async fn app() -> Router {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    tracing::info!("Connecting to database: {}", db_url);
    let mut opt = ConnectOptions::new(&db_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);

    let db: DatabaseConnection = Database::connect(opt)
        .await
        .expect("Database connection failed");

    let _ = Migrator::up(&db, None).await;

    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(MyContext::new(db))
        .finish();

    Router::new()
        .route("/graphql", post(graphql_handler).get(graphql_handler))
        .route("/graphiql", get(graphql_playground))
        .layer(Extension(schema))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::database::{
        handle_message_action, handle_user_action, MessageAction, UserAction,
    };
    use crate::entity::{message, user};
    use axum::{
        body::{to_bytes, Body},
        http::{self, Request, StatusCode},
    };

    use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
    use sea_orm::{ConnectionTrait, Database};
    use serde_json::{json, Value};
    use tower::ServiceExt;

    async fn setup_app() -> Router {
        dotenvy::dotenv().ok();
        let db_url = std::env::var("TEST_INTEGRATION_URL").expect("DATABASE_URL is not set");

        let db: DatabaseConnection = Database::connect(db_url)
            .await
            .expect("Database connection failed");

        load_test_data(&db).await;

        let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
            .data(MyContext::new(db))
            .finish();

        Router::new()
            .route("/graphql", post(graphql_handler).get(graphql_handler))
            .route("/graphiql", get(graphql_playground))
            .layer(Extension(schema))
    }

    async fn load_test_data(db: &DatabaseConnection) {
        // Reset the database
        user::Entity::delete_many()
            .filter(user::Column::Id.gt(0))
            .exec(db)
            .await
            .unwrap();
        message::Entity::delete_many()
            .filter(message::Column::Id.gt(0))
            .exec(db)
            .await
            .unwrap();
        // Reset auto increment
        let sql = "ALTER SEQUENCE user_id_seq RESTART WITH 1;";
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            sql.to_owned(),
        ))
        .await
        .expect("Could not reset auto increment");
        let sql = "ALTER SEQUENCE message_id_seq RESTART WITH 1;";
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            sql.to_owned(),
        ))
        .await
        .expect("Could not reset auto increment");

        let users = vec![
            ("Alice", 1),
            ("Bob", 2),
            ("Charlie", 3),
            ("David", 4),
            ("Eve", 5),
        ];

        // Create users sequentially
        for (name, _) in users {
            handle_user_action(db, UserAction::Create(name.to_string()))
                .await
                .unwrap();
        }

        let messages = vec![
            ("Hello, world!", 1),
            ("I am Alice", 1),
            ("Hi, there!", 2),
            ("How are you?", 3),
            ("I'm fine, thank you!", 4),
            ("Goodbye!", 5),
        ];

        // Create messages sequentially
        for (content, user_id) in messages {
            handle_message_action(db, MessageAction::Create(user_id, content.to_string()))
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn test_get_user_found() {
        let app = setup_app().await;
        let req = Request::builder()
            .uri("/graphql")
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"query":"{ getUser(id: 2) { id name } }"}"#))
            .unwrap();

        let response = app.oneshot(req).await.expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::OK);
        // Parse
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value,
            json!({
                "data": {
                    "getUser": {
                        "id": "2",
                        "name": "Bob"
                    }
                }
            })
        );
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let app = setup_app().await;
        let req = Request::builder()
            .uri("/graphql")
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"query":"{ getUser(id: 99999) { id name } }"}"#,
            ))
            .unwrap();

        let response = app.oneshot(req).await.expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        // Get the message from the error
        // Check if the "errors" field exists
        if let Some(errors) = value.get("errors") {
            for error in errors.as_array().unwrap() {
                if let Some(message) = error.get("message") {
                    assert_eq!(message, "User not found");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_update_user() {
        let app = setup_app().await;
        let req = Request::builder()
            .uri("/graphql")
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"query":"mutation { updateUser(id: 2, name: \"Bobby\") { success message } }"}"#))
            .unwrap();

        let response = app.oneshot(req).await.expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value,
            json!({
                "data": {
                      "updateUser": {
                    "success": true,
                    "message": "Action succeeded"
                }
                }
            })
        );
    }

    #[tokio::test]
    async fn test_delete_user() {
        let app = setup_app().await;
        let req = Request::builder()
            .uri("/graphql")
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"query":"mutation { deleteUser(id: 2) { success message } }"}"#,
            ))
            .unwrap();

        let response = app.oneshot(req).await.expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value,
            json!({
                "data": {
                      "deleteUser": {
                    "success": true,
                    "message": "Action succeeded"
                }
                }
            })
        );
    }

    #[tokio::test]
    async fn test_get_all_message() {
        let app = setup_app().await;
        let req = Request::builder()
            .uri("/graphql")
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"query":"{ getAllMessagesForUser (userId: 1) { userId content } }"}"#,
            ))
            .unwrap();
        let response = app.oneshot(req).await.expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value,
            json!({
                "data": {
                    "getAllMessagesForUser": [
                        {
                            "userId": "1",
                            "content": "Hello, world!"
                        },
                        {
                            "userId": "1",
                            "content": "I am Alice"
                        }
                    ]
                }
            })
        );
    }

    #[tokio::test]
    async fn test_delete_message() {
        let app = setup_app().await;
        let req = Request::builder()
            .uri("/graphql")
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"query":"mutation { deleteMessage(id: 1) { success message } }"}"#,
            ))
            .unwrap();

        let response = app.oneshot(req).await.expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value,
            json!({
                "data": {
                    "deleteMessage": {
                    "success": true,
                    "message": "Action succeeded"
                }
                }
            })
        );
    }

    #[tokio::test]
    async fn test_update_message() {
        let app = setup_app().await;
        let req = Request::builder()
            .uri("/graphql")
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"query":"mutation { updateMessage(id: 1, content: \"THIS IS AN UPDATED MESSAGE\") { success message } }"}"#,
            ))
            .unwrap();

        let response = app
            .clone()
            .oneshot(req)
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value,
            json!({
                "data": {
                    "updateMessage": {
                    "success": true,
                    "message": "Action succeeded"
                }
                }
            })
        );

        // Check if the message was updated
        let req = Request::builder()
            .uri("/graphql")
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"query":"{ getMessage(id: 1) { id content } }"}"#,
            ))
            .unwrap();
        let response = app.oneshot(req).await.expect("Failed to execute request");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value,
            json!({
                "data": {
                    "getMessage": {
                        "id": "1",
                        "content": "THIS IS AN UPDATED MESSAGE"
                    }
                }
            })
        );
    }

    #[tokio::test]
    async fn test_updated_non_existent_message() {
        let app = setup_app().await;
        let req = Request::builder()
            .uri("/graphql")
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"query":"mutation { updateMessage(id: 99999, content: \"THIS IS AN UPDATED MESSAGE\") { success message } }"}"#,
            ))
            .unwrap();

        let response = app
            .clone()
            .oneshot(req)
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value,
            json!({
                "data": {
                    "updateMessage": {
                    "success": true,
                    "message": "Action succeeded"
                }
                }
            })
        );

        let req = Request::builder()
            .uri("/graphql")
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"query":"{ getMessage(id: 99999) { id content } }"}"#,
            ))
            .unwrap();
        let response = app.oneshot(req).await.expect("Failed to execute request");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        if let Some(errors) = value.get("errors") {
            for error in errors.as_array().unwrap() {
                if let Some(message) = error.get("message") {
                    assert_eq!(message, "Message not found");
                }
            }
        }
    }

    #[tokio::test]
    async fn get_messages_in_time_range() {
        let app = setup_app().await;
        let req = Request::builder()
            .uri("/graphql")
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"query":"{ getMessagesInTimeRangeForUser(userId: 1, start: \"2021-01-01T00:00:00Z\", end: \"2099-01-02T00:00:00Z\") { id userId content } }"}"#))
            .unwrap();
        let response = app.oneshot(req).await.expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value,
            json!({
                "data": {
                    "getMessagesInTimeRangeForUser": [
                        {
                            "id": "1",
                            "userId": "1",
                            "content": "Hello, world!"
                        },
                        {
                            "id": "2",
                            "userId": "1",
                            "content": "I am Alice"
                        }
                    ]
                }
            })
        );
    }
}
