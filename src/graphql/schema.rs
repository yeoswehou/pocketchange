#![allow(dead_code)]
#![allow(unused_imports)]

use crate::db::database::{
    handle_message_action, handle_user_action, DatabaseAction, MessageAction, UserAction,
};
use crate::graphql::types::{Message, User};
use async_graphql::{Context, FieldError, FieldResult, Object, Schema, SimpleObject, ID};
use chrono::prelude::*;
use sea_orm::DatabaseConnection;
use tracing_subscriber::registry::Data;

pub struct MyContext {
    db: DatabaseConnection,
}

impl MyContext {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    pub async fn get_user(&self, ctx: &Context<'_>, id: ID) -> FieldResult<User> {
        let db = ctx.data_unchecked::<MyContext>().db.clone();
        let user_id = id.parse::<i32>()?;
        let action_result = handle_user_action(&db, UserAction::Get(user_id)).await?;

        match action_result {
            DatabaseAction::User(user) => Ok(User {
                id: ID(user.id.to_string()),
                name: user.name,
            }),
            DatabaseAction::Failure(message) => Err(async_graphql::Error::new(message)),
            _ => Err(async_graphql::Error::new("Unexpected database action")),
        }
    }

    pub async fn get_message(&self, ctx: &Context<'_>, id: ID) -> FieldResult<Option<Message>> {
        let db = ctx.data_unchecked::<MyContext>().db.clone();
        let message_id = id.parse::<i32>()?;
        let message = handle_message_action(&db, MessageAction::Get(message_id)).await?;

        match message {
            DatabaseAction::Message(message) => Ok(Some(Message {
                id: ID(message.id.to_string()),
                user_id: ID(message.user_id.to_string()),
                content: message.content,
                created_at: message.created_at,
                updated_at: message.updated_at,
                parent_id: message.parent_id,
            })),
            DatabaseAction::Failure(message) => Err(async_graphql::Error::new(message)),
            _ => Ok(None),
        }
    }

    // Resolver for fetching all messages for a specific user
    pub async fn get_all_messages_for_user(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
    ) -> FieldResult<Vec<Message>> {
        let db = ctx.data_unchecked::<MyContext>().db.clone();
        let uid = user_id.parse::<i32>()?;
        let messages = handle_message_action(&db, MessageAction::GetAllForUser(uid)).await?;
        match messages {
            DatabaseAction::Messages(messages) => Ok(messages
                .into_iter()
                .map(|msg| Message {
                    id: ID(msg.id.to_string()),
                    user_id: ID(msg.user_id.to_string()),
                    content: msg.content,
                    created_at: msg.created_at,
                    updated_at: msg.updated_at,
                    parent_id: msg.parent_id,
                })
                .collect()),
            _ => Err(async_graphql::Error::new("Failed to fetch messages")),
        }
    }

    pub async fn get_messages_in_time_range_for_user(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        start: String,
        end: String,
    ) -> FieldResult<Vec<Message>> {
        let db = ctx.data_unchecked::<MyContext>().db.clone();
        let uid = user_id.parse::<i32>()?;
        let start = DateTime::parse_from_rfc3339(&start)
            .map_err(|e| async_graphql::Error::new(format!("Invalid start datetime: {}", e)))?
            .with_timezone(&Utc);
        let end = DateTime::parse_from_rfc3339(&end)
            .map_err(|e| async_graphql::Error::new(format!("Invalid end datetime: {}", e)))?
            .with_timezone(&Utc);
        let result =
            handle_message_action(&db, MessageAction::GetInTimeRangeForUser(uid, start, end))
                .await?;

        match result {
            DatabaseAction::Messages(messages) => Ok(messages
                .into_iter()
                .map(|msg| Message {
                    id: ID(msg.id.to_string()),
                    user_id: ID(msg.user_id.to_string()),
                    content: msg.content,
                    created_at: msg.created_at,
                    updated_at: msg.updated_at,
                    parent_id: msg.parent_id,
                })
                .collect()),
            _ => Err(async_graphql::Error::new("Failed to fetch messages")),
        }
    }
}

pub type MySchema = Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;

// MutationRoot for creating user
pub struct MutationRoot;

#[derive(SimpleObject)]
pub struct MutationResponse {
    pub success: bool,
    pub message: String,
}

async fn handle_database_action(result: DatabaseAction) -> FieldResult<MutationResponse> {
    match result {
        DatabaseAction::Success => Ok(MutationResponse {
            success: true,
            message: "Action succeeded".to_string(),
        }),
        DatabaseAction::Failure(message) => Err(FieldError::new(message)),
        DatabaseAction::User(_) => Ok(MutationResponse {
            success: true,
            message: "User action succeeded".to_string(),
        }),
        DatabaseAction::Message(_) => Ok(MutationResponse {
            success: true,
            message: "Message action succeeded".to_string(),
        }),
        DatabaseAction::Messages(_) => Ok(MutationResponse {
            success: true,
            message: "Messages action succeeded".to_string(),
        }),
    }
}

#[Object]
impl MutationRoot {
    pub async fn create_user(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> FieldResult<MutationResponse> {
        let db = ctx.data_unchecked::<MyContext>().db.clone();
        let result = handle_user_action(&db, UserAction::Create(name)).await?;
        handle_database_action(result).await
    }

    pub async fn update_user(
        &self,
        ctx: &Context<'_>,
        id: ID,
        name: String,
    ) -> FieldResult<MutationResponse> {
        let db = ctx.data_unchecked::<MyContext>().db.clone();
        let user_id = id.parse::<i32>()?;
        let result = handle_user_action(&db, UserAction::Update(user_id, name)).await?;
        handle_database_action(result).await
    }

    pub async fn delete_user(&self, ctx: &Context<'_>, id: ID) -> FieldResult<MutationResponse> {
        let db = ctx.data_unchecked::<MyContext>().db.clone();
        let user_id = id.parse::<i32>()?;
        let result = handle_user_action(&db, UserAction::Delete(user_id)).await?;
        handle_database_action(result).await
    }

    pub async fn create_message(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        content: String,
        // parent_id: Option<i32>,
    ) -> FieldResult<MutationResponse> {
        let db = ctx.data_unchecked::<MyContext>().db.clone();
        let user_id = user_id.parse::<i32>()?;
        let result = handle_message_action(&db, MessageAction::Create(user_id, content)).await?;
        handle_database_action(result).await
    }

    pub async fn delete_message(&self, ctx: &Context<'_>, id: ID) -> FieldResult<MutationResponse> {
        let db = ctx.data_unchecked::<MyContext>().db.clone();
        let message_id = id.parse::<i32>()?;
        let result = handle_message_action(&db, MessageAction::Delete(message_id)).await?;
        handle_database_action(result).await
    }

    pub async fn update_message(
        &self,
        ctx: &Context<'_>,
        id: ID,
        content: String,
    ) -> FieldResult<MutationResponse> {
        let db = ctx.data_unchecked::<MyContext>().db.clone();
        let message_id = id.parse::<i32>()?;
        let result = handle_message_action(&db, MessageAction::Update(message_id, content)).await?;
        handle_database_action(result).await
    }
}
