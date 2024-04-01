use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod db;
pub mod entity;
pub mod graphql;
mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = server::app().await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!(
        "GraphiQL: http://localhost:{}/graphiql",
        listener.local_addr().unwrap().port()
    );

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
