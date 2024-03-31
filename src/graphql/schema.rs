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
}

pub type MySchema = Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;

// MutationRoot for creating user
pub struct MutationRoot;

#[derive(SimpleObject)]
pub struct MutationResponse {
    pub success: bool,
    pub message: String,
}

async fn handle_database_action(result: DatabaseAction) -> FieldResult<bool> {
    match result {
        DatabaseAction::Success => Ok(true),
        DatabaseAction::Failure(message) => Err(FieldError::new(message)),
        DatabaseAction::User(_) => Ok(true),
    }
}

#[Object]
impl MutationRoot {
    pub async fn create_user(&self, _ctx: &Context<'_>, name: String) -> FieldResult<bool> {
        let db = _ctx.data::<MyContext>().unwrap().db.clone();
        let success = handle_user_action(&db, UserAction::Create(name)).await?;
        handle_database_action(success).await
    }

    pub async fn update_user(&self, _ctx: &Context<'_>, id: ID, name: String) -> FieldResult<bool> {
        let db = _ctx.data::<MyContext>().unwrap().db.clone();
        let user_id = id.parse::<i32>().unwrap();
        let success = handle_user_action(&db, UserAction::Update(user_id, name)).await?;
        handle_database_action(success).await
    }

    pub async fn delete_user(&self, _ctx: &Context<'_>, id: ID) -> FieldResult<bool> {
        let db = _ctx.data::<MyContext>().unwrap().db.clone();
        let user_id = id.parse::<i32>().unwrap();
        let success = handle_user_action(&db, UserAction::Delete(user_id)).await?;
        handle_database_action(success).await
    }
}
