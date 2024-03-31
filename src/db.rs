#![allow(dead_code)]

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

use crate::entity::{message, user};

enum UserAction {
    Create(String),
    Delete(i32),
    Update(i32, String),
    Get(i32),
}

enum MessageAction {
    Created(i32, String),
    Update(i32, String),
    Delete(i32),
    TimeRange(i32, i32),
    Print(i32),
}

async fn handle_action(db: &DatabaseConnection, action: UserAction) -> Result<(), DbErr> {
    match action {
        UserAction::Create(name) => {
            create_user(db, &name).await?;
            println!("User created: {}", name);
        }
        UserAction::Delete(user_id) => {
            delete_user(db, user_id).await?;
            println!("User deleted: ID {}", user_id);
        }
        UserAction::Get(user_id) => {
            get_user(db, user_id).await?;
        }
        UserAction::Update(user_id, name) => {
            update_user(db, user_id, &name).await?;
            println!("User updated: ID {}, Name: {}", user_id, name);
        }
    }
    Ok(())
}

async fn create_user(db: &DatabaseConnection, name: &str) -> Result<(), DbErr> {
    let user = user::ActiveModel {
        name: Set(name.to_owned()),
        ..Default::default()
    };
    user.insert(db).await?;
    Ok(())
}

// Update the user's name
async fn update_user(db: &DatabaseConnection, user_id: i32, new_name: &str) -> Result<(), DbErr> {
    let filtered_user = user::Entity::find_by_id(user_id).one(db).await?;
    let mut mut_filtered_user: user::ActiveModel = filtered_user.unwrap().into();
    mut_filtered_user.name = Set(new_name.to_owned());
    mut_filtered_user.update(db).await?;
    Ok(())
}

async fn delete_user(db: &DatabaseConnection, user_id: i32) -> Result<(), DbErr> {
    user::Entity::delete_by_id(user_id).exec(db).await?;
    Ok(())
}

async fn get_user(db: &DatabaseConnection, user_id: i32) -> Result<Option<user::Model>, DbErr> {
    let user = user::Entity::find_by_id(user_id).one(db).await?;
    Ok(user)
}

async fn create_message(db: DatabaseConnection, user_id: i32, content: &str) -> Result<(), DbErr> {
    let message = message::ActiveModel {
        user_id: Set(user_id),
        content: Set(content.to_owned()),
        ..Default::default()
    };

    message.insert(&db).await?;
    Ok(())
}

// Print functions for users and messages for easier testing
async fn print_users(db: DatabaseConnection) {
    let users = user::Entity::find().all(&db).await.unwrap();
    println!("Users: {:?}", users);
}

async fn print_messages(db: DatabaseConnection, user_id: i32) {
    let messages = message::Entity::find()
        .filter(message::Column::UserId.eq(user_id))
        .all(&db)
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
}
