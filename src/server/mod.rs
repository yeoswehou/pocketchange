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

use crate::graphql::schema::{MutationRoot, MyContext, MySchema, QueryRoot};

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
