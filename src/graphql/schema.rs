use crate::graphql::types::{Message, User};
use async_graphql::{Context, Object, Schema, ID};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    pub async fn user(&self, _ctx: &Context<'_>) -> Option<User> {
        Some(User {
            id: ID("1".to_string()),
            name: "John Doe".to_string(),
        })
    }

    pub async fn message(&self, _ctx: &Context<'_>) -> Option<Message> {
        Some(Message {
            id: ID("1".to_string()),
            user: User {
                id: ID("1".to_string()),
                name: "John Doe".to_string(),
            },
            text: "Hello, world!".to_string(),
            timestamp: 1234567890,
        })
    }
}

pub type MySchema =
    Schema<QueryRoot, async_graphql::EmptyMutation, async_graphql::EmptySubscription>;
