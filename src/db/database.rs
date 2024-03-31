#![allow(dead_code)]

use crate::entity::{message, user};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

pub enum UserAction {
    Create(String),
    Delete(i32),
    Update(i32, String),
    Get(i32),
}

pub enum MessageAction {
    Created(i32, String),
    Update(i32, String),
    Delete(i32),
    TimeRange(i32, DateTime<Utc>, DateTime<Utc>),
    Print(i32),
}

pub enum DatabaseAction {
    Success,
    Failure(String),
    User(user::Model),
}

pub async fn handle_user_action(
    db: &DatabaseConnection,
    action: UserAction,
) -> Result<DatabaseAction, DbErr> {
    match action {
        UserAction::Create(name) => {
            create_user(db, &name).await?;
            println!("User created: {}", name);
            Ok(DatabaseAction::Success)
        }
        UserAction::Delete(user_id) => {
            delete_user(db, user_id).await?;
            println!("User deleted: ID {}", user_id);
            Ok(DatabaseAction::Success)
        }
        UserAction::Get(user_id) => {
            let user = get_user(db, user_id).await?;
            match user {
                Some(user) => {
                    println!("User found: ID {}, Name: {}", user.id, user.name);
                    Ok(DatabaseAction::User(user))
                }
                None => {
                    println!("User not found: ID {}", user_id);
                    Ok(DatabaseAction::Failure("User not found".to_string()))
                }
            }
        }
        UserAction::Update(user_id, name) => {
            update_user(db, user_id, &name).await?;
            println!("User updated: ID {}, Name: {}", user_id, name);
            Ok(DatabaseAction::Success)
        }
    }
}

async fn create_user(db: &DatabaseConnection, name: &str) -> Result<DatabaseAction, DbErr> {
    let user = user::ActiveModel {
        name: Set(name.to_owned()),
        ..Default::default()
    };
    user.insert(db).await?;
    Ok(DatabaseAction::Success)
}

// Update the user's name
async fn update_user(
    db: &DatabaseConnection,
    user_id: i32,
    new_name: &str,
) -> Result<DatabaseAction, DbErr> {
    let filtered_user = user::Entity::find_by_id(user_id).one(db).await?;
    if let Some(user) = filtered_user {
        let mut mut_filtered_user: user::ActiveModel = user.into();
        mut_filtered_user.name = Set(new_name.to_owned());
        mut_filtered_user.update(db).await?;
        Ok(DatabaseAction::Success)
    } else {
        println!("User not found: ID {}", user_id);
        Ok(DatabaseAction::Failure("User not found".to_string()))
    }
}

async fn delete_user(db: &DatabaseConnection, user_id: i32) -> Result<DatabaseAction, DbErr> {
    let result = user::Entity::delete_by_id(user_id).exec(db).await?;
    if result.rows_affected > 0 {
        Ok(DatabaseAction::Success)
    } else {
        println!("User not found: ID {}", user_id);
        Ok(DatabaseAction::Failure("User not found".to_string()))
    }
}

async fn get_user(db: &DatabaseConnection, user_id: i32) -> Result<Option<user::Model>, DbErr> {
    let user = user::Entity::find_by_id(user_id).one(db).await?;
    Ok(user)
}

// Print functions for users and messages for easier testing
async fn print_users(db: DatabaseConnection) {
    let users = user::Entity::find().all(&db).await.unwrap();
    println!("Users: {:?}", users);
}

pub async fn handle_message_action(
    db: &DatabaseConnection,
    action: MessageAction,
) -> Result<(), DbErr> {
    match action {
        MessageAction::Created(user_id, content) => {
            create_message(db, user_id, &content).await?;
            println!("Message created: User ID {}, Content: {}", user_id, content);
        }
        MessageAction::Update(message_id, content) => {
            update_message(db, message_id, &content).await?;
            println!("Message updated: ID {}, Content: {}", message_id, content);
        }
        MessageAction::Delete(message_id) => {
            delete_message(db, message_id).await?;
            println!("Message deleted: ID {}", message_id);
        }
        MessageAction::TimeRange(user_id, start, end) => {
            get_messages_in_time_range(db, user_id, start, end)
                .await
                .expect("No messages found in time range");
        }
        MessageAction::Print(user_id) => {
            print_messages(db, user_id).await;
        }
    }
    Ok(())
}

async fn create_message(db: &DatabaseConnection, user_id: i32, content: &str) -> Result<(), DbErr> {
    let message = message::ActiveModel {
        user_id: Set(user_id),
        content: Set(content.to_owned()),
        ..Default::default()
    };

    message.insert(db).await?;
    Ok(())
}

async fn update_message(
    db: &DatabaseConnection,
    message_id: i32,
    new_content: &str,
) -> Result<(), DbErr> {
    let filtered_message = message::Entity::find_by_id(message_id).one(db).await?;
    let mut mut_filtered_message: message::ActiveModel = filtered_message.unwrap().into();
    mut_filtered_message.content = Set(new_content.to_owned());
    mut_filtered_message.update(db).await?;
    Ok(())
}

async fn delete_message(db: &DatabaseConnection, message_id: i32) -> Result<(), DbErr> {
    message::Entity::delete_by_id(message_id).exec(db).await?;
    Ok(())
}

async fn get_messages_in_time_range(
    db: &DatabaseConnection,
    user_id: i32,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<message::Model>, DbErr> {
    let messages = message::Entity::find()
        .filter(message::Column::UserId.eq(user_id))
        .filter(message::Column::CreatedAt.between(start, end))
        .all(db)
        .await?;

    println!("Messages in time range: {:?}", messages);
    Ok(messages)
}

async fn print_messages(db: &DatabaseConnection, user_id: i32) {
    let messages = message::Entity::find()
        .filter(message::Column::UserId.eq(user_id))
        .all(db)
        .await
        .unwrap();
    println!("Messages: {:?}", messages);
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;
    use sea_orm::Database;
    use std::env;
    use tokio::time;

    async fn setup() -> DatabaseConnection {
        dotenv().ok();
        let db_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        let db = Database::connect(&db_url)
            .await
            .expect("Failed to connect to test database");
        user::Entity::delete_many()
            .filter(user::Column::Id.gt(0))
            .exec(&db)
            .await
            .unwrap();
        db
    }

    #[tokio::test]
    async fn test_create_user() {
        let db = setup().await;
        let name = "Alice";
        let new_user_result = create_user(&db, name).await;
        assert!(new_user_result.is_ok(), "Failed to create user");

        let user = user::Entity::find()
            .filter(user::Column::Name.eq(name))
            .one(&db)
            .await
            .expect("Failed to find user");
        assert!(user.is_some(), "User not found");
        assert_eq!(user.unwrap().name, name);
    }

    #[tokio::test]
    async fn test_update_user() {
        let db = setup().await;
        let name = "Bob";
        let new_name = "Carl";
        create_user(&db, name).await.expect("Failed to create user");

        let user = user::Entity::find()
            .filter(user::Column::Name.eq(name))
            .one(&db)
            .await
            .expect("Failed to find user")
            .expect("User not found");

        let user_id = user.id;
        update_user(&db, user_id, new_name)
            .await
            .expect("Failed to update user");

        let updated_user = user::Entity::find_by_id(user_id)
            .one(&db)
            .await
            .expect("Failed to find user")
            .expect("User not found");

        assert_eq!(updated_user.name, new_name);
    }

    #[tokio::test]
    async fn test_delete_user() {
        let db = setup().await;
        let name = "Dave";
        create_user(&db, name).await.expect("Failed to create user");

        let user = user::Entity::find()
            .filter(user::Column::Name.eq(name))
            .one(&db)
            .await
            .expect("Failed to find user")
            .expect("User not found");

        let user_id = user.id;
        delete_user(&db, user_id)
            .await
            .expect("Failed to delete user");

        let deleted_user = user::Entity::find_by_id(user_id)
            .one(&db)
            .await
            .expect("Failed to find user");

        assert!(deleted_user.is_none(), "User not deleted");
    }

    #[tokio::test]
    async fn test_get_user() {
        let db = setup().await;
        let name = "Igor";
        create_user(&db, name).await.expect("Failed to create user");

        let user = user::Entity::find()
            .filter(user::Column::Name.eq(name))
            .one(&db)
            .await
            .expect("Failed to find user")
            .expect("User not found");

        let user_id = user.id;
        let found_user = get_user(&db, user_id).await.expect("Failed to get user");
        assert!(found_user.is_some(), "User not found");
        assert_eq!(found_user.unwrap().name, name);
    }

    #[tokio::test]
    async fn test_create_message() {
        let db = setup().await;
        let user_name = "Alex";
        let message_content = "Hello, world!";
        create_user(&db, user_name)
            .await
            .expect("Failed to create user");

        let user = user::Entity::find()
            .filter(user::Column::Name.eq(user_name))
            .one(&db)
            .await
            .expect("Failed to find user")
            .expect("User not found");

        let user_id = user.id;
        create_message(&db, user_id, message_content)
            .await
            .expect("Failed to create message");

        let message = message::Entity::find()
            .filter(message::Column::UserId.eq(user_id))
            .one(&db)
            .await
            .expect("Failed to find message")
            .expect("Message not found");

        assert_eq!(message.content, message_content);
    }

    #[tokio::test]
    async fn test_update_message() {
        let db = setup().await;
        let user_name = "Bob Dylan";
        let message_content = "Hello, world!";
        create_user(&db, user_name)
            .await
            .expect("Failed to create user");

        let user = user::Entity::find()
            .filter(user::Column::Name.eq(user_name))
            .one(&db)
            .await
            .expect("Failed to find user")
            .expect("User not found");

        let user_id = user.id;
        create_message(&db, user_id, message_content)
            .await
            .expect("Failed to create message");

        let message = message::Entity::find()
            .filter(message::Column::UserId.eq(user_id))
            .one(&db)
            .await
            .expect("Failed to find message")
            .expect("Message not found");

        let message_id = message.id;
        let new_content = "Goodbye, world!";
        update_message(&db, message_id, new_content)
            .await
            .expect("Failed to update message");

        let updated_message = message::Entity::find_by_id(message_id)
            .one(&db)
            .await
            .expect("Failed to find message")
            .expect("Message not found");

        assert_eq!(updated_message.content, new_content);
    }

    #[tokio::test]
    async fn test_delete_message() {
        let db = setup().await;
        let user_name = "Charlie";
        let message_content = "Hello, world!";
        create_user(&db, user_name)
            .await
            .expect("Failed to create user");

        let user = user::Entity::find()
            .filter(user::Column::Name.eq(user_name))
            .one(&db)
            .await
            .expect("Failed to find user")
            .expect("User not found");

        let user_id = user.id;
        create_message(&db, user_id, message_content)
            .await
            .expect("Failed to create message");

        let message = message::Entity::find()
            .filter(message::Column::UserId.eq(user_id))
            .one(&db)
            .await
            .expect("Failed to find message")
            .expect("Message not found");

        let message_id = message.id;
        delete_message(&db, message_id)
            .await
            .expect("Failed to delete message");

        let deleted_message = message::Entity::find_by_id(message_id)
            .one(&db)
            .await
            .expect("Failed to find message");

        assert!(deleted_message.is_none(), "Message not deleted");
    }

    #[tokio::test]
    async fn test_get_messages_in_time_range() {
        let db = setup().await;
        let user_name = "David";
        let message_content = "Hello, world!";
        create_user(&db, user_name)
            .await
            .expect("Failed to create user");

        let user = user::Entity::find()
            .filter(user::Column::Name.eq(user_name))
            .one(&db)
            .await
            .expect("Failed to find user")
            .expect("User not found");

        let user_id = user.id;
        for _ in 0..10 {
            create_message(&db, user_id, message_content)
                .await
                .expect("Failed to create message");
        }
        time::sleep(time::Duration::from_secs(1)).await;

        let start = chrono::Utc::now() - chrono::Duration::days(1);
        let end = chrono::Utc::now();
        let messages = get_messages_in_time_range(&db, user_id, start, end).await;
        let result = messages.expect("Failed to get messages in time range");
        assert_eq!(result.len(), 10);
    }
}
