use async_graphql::{Object, ID};

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
    pub text: String,
    pub timestamp: i64,
}

#[Object]
impl Message {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn user_id(&self) -> &ID {
        &self.user_id
    }

    async fn text(&self) -> &str {
        &self.text
    }

    async fn timestamp(&self) -> i64 {
        self.timestamp
    }
}
