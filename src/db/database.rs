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
    Create(i32, String),
    Get(i32),
    GetAllForUser(i32),
    GetInTimeRangeForUser(i32, DateTime<Utc>, DateTime<Utc>),
    Update(i32, String),
    Delete(i32),
}

pub enum DatabaseAction {
    Success,
    Failure(String),
    User(user::Model),
    Message(message::Model),
    Messages(Vec<message::Model>),
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

pub async fn handle_message_action(
    db: &DatabaseConnection,
    action: MessageAction,
) -> Result<DatabaseAction, DbErr> {
    match action {
        MessageAction::Create(user_id, content) => {
            create_message(db, user_id, &content).await?;
            println!("Message created: User ID {}, Content: {}", user_id, content);
            Ok(DatabaseAction::Success)
        }
        MessageAction::Get(message_id) => {
            let message = get_message(db, message_id).await?;
            match message {
                Some(message) => {
                    println!(
                        "Message found: ID {}, User ID {}, Content: {}",
                        message.id, message.user_id, message.content
                    );
                    Ok(DatabaseAction::Message(message))
                }
                None => {
                    println!("Message not found: ID {}", message_id);
                    Ok(DatabaseAction::Failure("Message not found".to_string()))
                }
            }
        }
        MessageAction::Update(message_id, content) => {
            update_message(db, message_id, &content).await?;
            println!("Message updated: ID {}, Content: {}", message_id, content);
            Ok(DatabaseAction::Success)
        }
        MessageAction::Delete(message_id) => {
            delete_message(db, message_id).await?;
            println!("Message deleted: ID {}", message_id);
            Ok(DatabaseAction::Success)
        }
        MessageAction::GetAllForUser(user_id) => {
            let messages = get_all_messages_for_user(db, user_id).await?;
            Ok(DatabaseAction::Messages(messages))
        }
        MessageAction::GetInTimeRangeForUser(user_id, start, end) => {
            let messages = get_messages_in_time_range(db, user_id, start, end).await?;
            Ok(DatabaseAction::Messages(messages))
        }
    }
}

async fn create_message(
    db: &DatabaseConnection,
    user_id: i32,
    content: &str,
) -> Result<DatabaseAction, DbErr> {
    let message = message::ActiveModel {
        user_id: Set(user_id),
        content: Set(content.to_owned()),
        ..Default::default()
    };

    message.insert(db).await?;
    Ok(DatabaseAction::Success)
}

async fn get_message(
    db: &DatabaseConnection,
    message_id: i32,
) -> Result<Option<message::Model>, DbErr> {
    let message = message::Entity::find_by_id(message_id).one(db).await?;
    Ok(message)
}

async fn update_message(
    db: &DatabaseConnection,
    message_id: i32,
    new_content: &str,
) -> Result<DatabaseAction, DbErr> {
    let filtered_message = message::Entity::find_by_id(message_id).one(db).await?;
    if let Some(filtered_message) = filtered_message {
        let mut mut_filtered_message: message::ActiveModel = filtered_message.into();
        mut_filtered_message.content = Set(new_content.to_owned());
        mut_filtered_message.update(db).await?;
        Ok(DatabaseAction::Success)
    } else {
        println!("Message not found: ID {}", message_id);
        Ok(DatabaseAction::Failure("Message not found".to_string()))
    }
}

async fn delete_message(db: &DatabaseConnection, message_id: i32) -> Result<DatabaseAction, DbErr> {
    let result = message::Entity::delete_by_id(message_id).exec(db).await?;
    if result.rows_affected > 0 {
        Ok(DatabaseAction::Success)
    } else {
        println!("Message not found: ID {}", message_id);
        Ok(DatabaseAction::Failure("Message not found".to_string()))
    }
}

async fn get_all_messages_for_user(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<Vec<message::Model>, DbErr> {
    let messages = message::Entity::find()
        .filter(message::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    Ok(messages)
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

    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;
    use migration::{Migrator, MigratorTrait};
    use sea_orm::{Database, TransactionTrait};
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
        Migrator::up(&db, None)
            .await
            .expect("Failed to apply migrations");
        db
    }

    #[tokio::test]
    async fn test_create_user() {
        let db = setup().await;
        let transaction = db.begin().await.expect("Failed to start transaction");
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
        transaction
            .rollback()
            .await
            .expect("Failed to rollback transaction");
    }

    #[tokio::test]
    async fn test_update_user() {
        let db = setup().await;
        let transaction = db.begin().await.expect("Failed to start transaction");
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
        transaction
            .rollback()
            .await
            .expect("Failed to rollback transaction");
    }

    #[tokio::test]
    async fn test_delete_user() {
        let db = setup().await;
        let transaction = db.begin().await.expect("Failed to start transaction");
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
        transaction
            .rollback()
            .await
            .expect("Failed to rollback transaction");
    }

    #[tokio::test]
    async fn test_get_user() {
        let db = setup().await;
        let transaction = db.begin().await.expect("Failed to start transaction");
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
        transaction
            .rollback()
            .await
            .expect("Failed to rollback transaction");
    }

    #[tokio::test]
    async fn test_create_message() {
        let db = setup().await;
        let transaction = db.begin().await.expect("Failed to start transaction");
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
        transaction
            .rollback()
            .await
            .expect("Failed to rollback transaction");
    }

    #[tokio::test]
    async fn test_update_message() {
        let db = setup().await;
        let transaction = db.begin().await.expect("Failed to start transaction");
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
        transaction
            .rollback()
            .await
            .expect("Failed to rollback transaction");
    }

    #[tokio::test]
    async fn test_delete_message() {
        let db = setup().await;
        let transaction = db.begin().await.expect("Failed to start transaction");
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
        transaction
            .rollback()
            .await
            .expect("Failed to rollback transaction");
    }

    #[tokio::test]
    async fn test_get_messages_in_time_range() {
        if env::var("CI").is_ok() {
            println!("Skipping this test on GitHub Actions.");
            return;
        }
        let db = setup().await;
        let transaction = db.begin().await.expect("Failed to start transaction");
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
        let end = chrono::Utc::now() + chrono::Duration::days(1);
        let messages = get_messages_in_time_range(&db, user_id, start, end).await;
        let result = messages.expect("Failed to get messages in time range");
        match result.len() {
            0 => panic!("No messages found in time range"),
            1 => assert_eq!(result.len(), 1),
            _ => assert!(result.len() > 1),
        }
        transaction
            .rollback()
            .await
            .expect("Failed to rollback transaction");
    }
}
