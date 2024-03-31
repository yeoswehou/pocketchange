use async_graphql::{Object, ID};
use chrono::{DateTime, Utc};

pub struct User {
    pub id: ID,
    pub name: String,
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

pub struct Message {
    pub id: ID,
    pub user_id: ID,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub parent_id: Option<i32>,
}

#[Object]
impl Message {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn user_id(&self) -> &ID {
        &self.user_id
    }
    async fn content(&self) -> &str {
        &self.content
    }

    async fn created_at(&self) -> String {
        self.created_at.to_rfc3339()
    }

    async fn updated_at(&self) -> String {
        self.updated_at.to_rfc3339()
    }

    async fn parent_id(&self) -> Option<i32> {
        self.parent_id
    }
}
