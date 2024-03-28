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

mod graphql;

use graphql::schema::{MySchema, QueryRoot};

async fn graphql_handler(schema: Extension<MySchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    let html = playground_source(GraphQLPlaygroundConfig::new("/graphql"));
    Html(html)
}

#[tokio::main]
async fn main() {
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
