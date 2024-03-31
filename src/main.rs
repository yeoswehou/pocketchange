use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::Html,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    Set,
};

use entity::{message, user};
use graphql::schema::{MySchema, QueryRoot};

mod entity;
mod graphql;

async fn graphql_handler(schema: Extension<MySchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    let html = playground_source(GraphQLPlaygroundConfig::new("/graphql"));
    Html(html)
}

async fn create_user(db: DatabaseConnection, name: &str) -> Result<user::Model, DbErr> {
    let user = user::ActiveModel {
        name: Set(name.to_owned()),
        ..Default::default()
    };

    let res = user.insert(&db).await?;
    Ok(res)
}

async fn print_users(db: DatabaseConnection) {
    let users = user::Entity::find().all(&db).await.unwrap();
    println!("Users: {:?}", users);
}

async fn create_message(db: DatabaseConnection, user_id: i32, content: &str) -> Result<(), DbErr> {
    let message = message::ActiveModel {
        user_id: Set(user_id),
        content: Set(content.to_owned()),
        ..Default::default()
    };

    message.insert(&db).await?;
    Ok(())
}

async fn print_messages(db: DatabaseConnection, user_id: i32) {
    let messages = message::Entity::find()
        .filter(message::Column::UserId.eq(user_id))
        .all(&db)
        .await
        .unwrap();
    println!("Messages: {:?}", messages);
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    println!("Connecting to database: {}", db_url);
    let db: DatabaseConnection = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    let new_user = create_user(db.clone(), "King")
        .await
        .expect("Failed to create user");
    println!("Created new user: {:?}", new_user);
    print_users(db.clone()).await;
    create_message(db.clone(), new_user.id, "Hello, world!")
        .await
        .expect("Failed to create message");
    print_messages(db.clone(), new_user.id).await;

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let app = Router::new()
        .route("/graphql", post(graphql_handler).get(graphql_handler))
        .route("/graphiql", get(graphql_playground))
        .layer(Extension(schema));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap();
    println!(
        "GraphiQL: http://localhost:{}/graphiql",
        listener.local_addr().unwrap().port()
    );
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
