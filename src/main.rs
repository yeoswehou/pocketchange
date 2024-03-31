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
use sea_orm::{Database, DatabaseConnection};

use graphql::schema::{MySchema, QueryRoot};

mod db;
mod entity;
mod graphql;

async fn graphql_handler(schema: Extension<MySchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    let html = playground_source(GraphQLPlaygroundConfig::new("/graphql"));
    Html(html)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    println!("Connecting to database: {}", db_url);
    let _db: DatabaseConnection = Database::connect(db_url)
        .await
        .expect("Database connection failed");

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
