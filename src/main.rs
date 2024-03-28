use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, ID};

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};

async fn graphql_playground() -> impl axum::response::IntoResponse {
    let html = playground_source(GraphQLPlaygroundConfig::new("/graphql"));
    axum::response::Html(html)
}

struct User {
    id: ID,
    name: String,
}

#[Object]
impl User {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn user(&self, _ctx: &Context<'_>) -> Option<User> {
        Some(User {
            id: ID("1".to_string()),
            name: "John Doe".to_string(),
        })
    }
}

type MySchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

async fn graphql_handler(schema: Extension<MySchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[tokio::main]
async fn main() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let app = Router::new()
        .route("/graphql", post(graphql_handler).get(graphql_handler)) // GraphQL endpoint
        .route("/graphiql", get(graphql_playground)) // GraphiQL interface endpoint
        .layer(Extension(schema));

    println!("GraphiQL playground at http://localhost:3000/graphiql");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
