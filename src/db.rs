#![allow(dead_code)]
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

use crate::entity::{message, user};

enum UserAction {
    CreateUser(String),
    DeleteUser(i32),
    UpdateUser(i32, String),
    PrintUsers,
}

enum MessageAction {
    CreateMessage(i32, String),
    UpdateMessage(i32, String),
    DeleteMessage(i32),
    TimeRangeMessages(i32, i32),
    PrintMessages(i32),
}

async fn handle_action(db: &DatabaseConnection, action: UserAction) -> Result<(), DbErr> {
    match action {
        UserAction::CreateUser(name) => {
            create_user(db, &name).await?;
            println!("User created: {}", name);
        }
        UserAction::DeleteUser(user_id) => {
            delete_user(db, user_id).await?;
            println!("User deleted: ID {}", user_id);
        }
        UserAction::PrintUsers => {
            let users = get_users(db).await?;
            for user in users {
                println!("User: {}", user.name);
            }
        }
        UserAction::UpdateUser(user_id, name) => {
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

async fn get_users(db: &DatabaseConnection) -> Result<Vec<user::Model>, DbErr> {
    let users = user::Entity::find().all(db).await?;
    Ok(users)
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
