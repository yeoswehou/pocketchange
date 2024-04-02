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
use sea_orm::{Database, DatabaseConnection};

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

    let db: DatabaseConnection = Database::connect(db_url)
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
    use futures::future::join_all;
    use sea_orm::Database;
    use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use tower::ServiceExt;

    async fn setup_app() -> Router {
        dotenvy::dotenv().ok();
        let db_url = std::env::var("TEST_DATABASE_URL").expect("DATABASE_URL is not set");

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

        let mut users = HashMap::new();
        users.insert("Alice", 1);
        users.insert("Bob", 2);
        users.insert("Charlie", 3);
        users.insert("David", 4);
        users.insert("Eve", 5);
        let user_futures: Vec<_> = users
            .keys()
            .map(|name| handle_user_action(db, UserAction::Create(name.to_string())))
            .collect();

        join_all(user_futures).await;

        let mut messages = HashMap::new();
        messages.insert("Hello, world!", 1);
        messages.insert("I am Alice", 1);
        messages.insert("Hi, there!", 2);
        messages.insert("How are you?", 3);
        messages.insert("I'm fine, thank you!", 4);
        messages.insert("Goodbye!", 5);
        let message_futures: Vec<_> = messages
            .iter()
            .map(|(content, user_id)| {
                handle_message_action(db, MessageAction::Create(*user_id, content.to_string()))
            })
            .collect();

        join_all(message_futures).await;
    }
    // 
    // #[tokio::test]
    // async fn test_get_user_found() {
    //     let app = setup_app().await;
    //     let req = Request::builder()
    //         .uri("/graphql")
    //         .method(http::Method::POST)
    //         .header(http::header::CONTENT_TYPE, "application/json")
    //         .body(Body::from(r#"{"query":"{ getUser(id: 2) { id name } }"}"#))
    //         .unwrap();
    // 
    //     let response = app.oneshot(req).await.expect("Failed to execute request");
    // 
    //     assert_eq!(response.status(), StatusCode::OK);
    //     // Parse
    //     let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    //     let value: Value = serde_json::from_slice(&body).unwrap();
    //     assert_eq!(
    //         value,
    //         json!({
    //             "data": {
    //                 "getUser": {
    //                     "id": 2,
    //                     "name": "Bob"
    //                 }
    //             }
    //         })
    //     );
    // }
}
