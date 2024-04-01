# pocketchange

## Requirements
Requirements: Please create a simple Rust web server:
* API should be provided as a GraphQL endpoint.
* We need two entities: User and Message.
* Each Message is written by the user and can be edited later.
* GraphQL endpoint should provide ways to create/modify/delete messages.
* GraphQL endpoint should provide a way to query messages by user and time range.
* Data should be stored in the database of your choice.
* (advanced. optional) Each message can be replied to. and it can be nested (reply -> reply -> reply ... should support)
* (advanced. optional) Each user can be registered/login by ID/password. Only the author can modify/delete the comments (the user should be identified by something. Authorization header or so on?)

## Libraries
- async-graphql
- Axum
- PostgresSQL
- SeaORM

## Project Structure

```plaintext

├── .github 
│   ├── workflows
│       ├── rust.yml  # CI/CD GitHub Actions
├── Cargo.lock
├── Cargo.toml
├── Dockerfile
├── README.md
├── docker-compose.yml  # Setup for PostgreSQL and Test Database
├── entrypoint.sh
├── migration  # Migration using SeaORM
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── README.md
│   └── src
│       ├── lib.rs
│       ├── m20220101_000001_create_table.rs
│       └── main.rs
└── src
    ├── db  # PostgreSQL Connection
    │   ├── database.rs
    │   └── mod.rs
    ├── entity  # SeaORM Entities
    │   ├── message.rs
    │   ├── mod.rs
    │   └── user.rs
    ├── graphql  # GraphQL Schema
    │   ├── mod.rs
    │   ├── schema.rs
    │   └── types.rs
    └── main.rs
```

## How to run
This will build the app and start the server.
```
docker-compose up app
```

Alternatively, you can run the following commands:
```
docker-compose up -d db
cargo run
```

## How to test
```
docker-compose up -d test-db
cargo test
```

## Environment Variables
Stored in a .env file because this is an assignment. It will be handled differently in production (GitHub Secrets etc.).

## Docker Commands for setup
Run the following command to start the database for use
```
docker-compose up -d db
```
Run the following command to connect to the database for testing
```
docker-compose up -d test-db
```
Run the following command to connect to the database for testing
```
docker-compose exec db psql -U username -d pocketchange
```
Run the following command to remove the database
```
 docker-compose down -v
```



# Access GraphQL Playground
It dynamically scans for an available port. The port is printed to the console.
The app will be available at http://localhost:PORT/graphql . 


# User Mutations
- **getUser**
```graphql
query {
  getUser(id: 2) {
    name
  }
}
```

**Sample Response** 

Success
```json
{
  "data": {
    "getUser": {
      "name": "helllo"
    }
  }
}
```

Failure (no such user)
```json
{
  "data": null,
  "errors": [
    {
      "message": "User not found",
      "locations": [
        {
          "line": 2,
          "column": 3
        }
      ],
      "path": [
        "getUser"
      ]
    }
  ]
}
```

- **createUser**
```graphql
mutation {
  createUser(name: "Peter") {
    success
    message
  }
} 
```

**Sample Response**

Success
```json
{
  "data": {
    "createUser": {
      "success": true,
      "message": "Action succeeded"
    }
  }
}
```

- **updateUser**
```graphql
mutation {
    updateUser(id: 1, name: "Peter Parker") {
    success
    message
  }
}
```

**Sample Response**

Success
```json
{
  "data": {
    "updateUser": {
      "success": true,
      "message": "Action succeeded"
    }
  }
}
```

- **deleteUser**
```graphql
mutation {
  deleteUser(id: 1) {
    success
    message
  }
}
```

**Sample Response**

Success
```json
{
  "data": {
    "deleteUser": {
      "success": true,
      "message": "Action succeeded"
    }
  }
}
```

## Message Mutations
- **createMessage**
```graphql
mutation {
  createMessage(userId: 9, content: "I am Batman") {
  success
  }
}
```

**Sample Response**

Success
```json
{
  "data": {
    "createMessage": {
      "success": true
    }
  }
}
```

- **updateMessage**
```graphql
mutation {
  updateMessage(id: 21, content: "I am not Batman") {
  success
  }
}
```

**Sample Response**

Success
```json
{
  "data": {
    "updateMessage": {
      "success": true
    }
  }
}
```

- **deleteMessage**
```graphql
mutation {
  deleteMessage(id: 22) {
    success
    message
  }
}
```

**Sample Response**

Success
```json
{
  "data": {
    "deleteMessage": {
      "success": true,
      "message": "Action succeeded"
    }
  }
}
```

- getMessagesByUser
```graphql
query {
  getAllMessagesForUser(userId:9) {
    id
    userId
    createdAt
    updatedAt
    content
  }
}
```

**Sample Response**

Success
```json
{
  "data": {
    "getAllMessagesForUser": [
      {
        "id": "21",
        "userId": "9",
        "createdAt": "2024-03-31T17:46:33.139078+00:00",
        "updatedAt": "2024-03-31T17:46:33.139078+00:00",
        "content": "I am Batman"
      }
    ]
  }
}
```

- getMessagesByTimeRange
```graphql
```

**Sample Response**

Success

## Checklist
- [x] Database (sqlx)
  - [X] User entity
  - [X] Message entity
  - [X] Create tables
  - [X] Query for User
  - [X] Query for Message
  - Test
- [X] GraphQL (async-graphql)
  - [X] Endpoint
  - [X] Schema
  - [X] Query
  - [X] Mutation
- [X] CRUD for Message
- [X] Query for Message by user and time range
- [ ] Advanced: Reply to Message
- [ ] Advanced: User registration/login
