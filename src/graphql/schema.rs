use crate::graphql::types::{Message, User};
use async_graphql::{Context, FieldResult, Object, Schema, ID};
use chrono::prelude::*;
use uuid::Uuid;

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
            user_id: ID("1".to_string()),
            text: "Hello, world!".to_string(),
            timestamp: 1234567890,
        })
    }
}

pub type MySchema =
    Schema<QueryRoot, async_graphql::EmptyMutation, async_graphql::EmptySubscription>;

// MutationRoot for creating user
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    pub async fn create_user(&self, _ctx: &Context<'_>, name: String) -> User {
        User {
            id: ID(Uuid::new_v4().to_string()),
            name,
        }
    }

    pub async fn delete_user(&self, _ctx: &Context<'_>, _id: ID) -> bool {
        // Temporary implementation
        true
    }

    pub async fn create_message(
        &self,
        _ctx: &Context<'_>,
        user_id: ID,
        text: String,
    ) -> FieldResult<Message> {
        let timestamp = Utc::now().timestamp();
        Ok(Message {
            id: ID(Uuid::new_v4().to_string()),
            user_id,
            text,
            timestamp,
        })
    }
}
