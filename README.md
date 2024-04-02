# Axum-GraphQL-SeaORM 

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

## Rust Version
Tested with
- rustc 1.77.1 (7cf61ebde 2024-03-27)


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

## How to Run
To build the app and start the server, run the following commands:
```bash
docker-compose build
docker-compose up app
```

Alternatively, you can start the database container separately and run the application using Cargo:
```bash
docker-compose up -d db
cargo run
```

## Load Sample Data
To load sample data, run the following command:
```bash
bash populate.sh
```

## How to Test
Needs SeaORM CLI to run the migrations for the test. 

Install it using the following command:
```bash
cargo install sea-orm-cli
```
To run the tests, use the following commands:
For simple testing, I created two databases for testing and integration testing. Currently, the tests are running sequentially, but it can be improved to run in parallel.
```bash
docker-compose up -d test-db
docker-compose up -d test-integration
sea-orm-cli migrate up -u postgresql://username:password@localhost:5433/pocketchangetest
sea-orm-cli migrate up -u postgresql://username:password@localhost:5434/pocketchangeintegration
cargo test -- --test-threads=1
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
I will use port 8080 for the GraphQL endpoint for easy access. (could be dynamic in production) 
The app will be available at http://localhost:8080/graphql . 

# GraphQL Schema
```graphql
schema {
  query: QueryRoot
  mutation: MutationRoot
}

type Message {
  id: ID!
  userId: ID!
  content: String!
  createdAt: String!
  updatedAt: String!
  parentId: Int
  user: User!
}

type MutationResponse {
  success: Boolean!
  message: String!
}

type MutationRoot {
  createUser(name: String!): MutationResponse!
  updateUser(id: ID!, name: String!): MutationResponse!
  deleteUser(id: ID!): MutationResponse!
  createMessage(userId: ID!, content: String!, parentId: Int): MutationResponse!
  deleteMessage(id: ID!): MutationResponse!
  updateMessage(id: ID!, content: String!): MutationResponse!
}

type QueryRoot {
  getUser(id: ID!): User!
  getMessage(id: ID!): Message
  getAllMessagesForUser(userId: ID!): [Message!]!
  getMessagesInTimeRangeForUser(
    userId: ID!
    start: String!
    end: String!
  ): [Message!]!
  getMessageThread(messageId: Int!): [Message!]!
}

type User {
  id: ID!
  name: String!
}
```

# User Mutations
- **getUser**
```graphql
query {
  getUser(id: 2) {
    name
  }
}
```

```shell
curl -X POST -H "Content-Type: application/json" -d '{"query": "query { getUser(id: 2) { name } }"}' http://localhost:8080/graphql
```

**Sample Response** 

Success
```json
{
  "data": {
    "getUser": {
      "name": "User 2"
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
```shell
curl -X POST -H "Content-Type: application/json" -d '{"query": "mutation { createUser(name: \"Superman\") { success message } }"}' http://localhost:8080/graphql
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

```shell
curl -X POST -H "Content-Type: application/json" -d '{"query": "mutation { updateUser(id: 1, name: \"Peter Parker\") { success message } }"}' http://localhost:8080/graphql
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

```shell
curl -X POST -H "Content-Type: application/json" -d '{"query": "mutation { deleteUser(id: 1) { success message } }"}' http://localhost:8080/graphql
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
**- NO NESTING**
```graphql
mutation {
  createMessage(userId: 1, content: "I am Batman") {
  success
  }
}
```
```shell
curl -X POST -H "Content-Type: application/json" -d '{"query": "mutation { createMessage(userId: 1, content: \"I am Batman\") { success } }"}' http://localhost:8080/graphql
```

**- NESTING**
```graphql
mutation {
  createMessage(userId: 1, content: "I am Batman", parentId: 2) {
  success
  }
}
```
```shell
curl -X POST -H "Content-Type: application/json" \
     -d '{"query": "mutation { createMessage(userId: 1, content: \"I am Batman\", parentId: 2) { success } }"}' \
     http://localhost:8080/graphql
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
  updateMessage(id: 1, content: "I am not Batman") {
  success
  }
}
```

```shell
curl -X POST -H "Content-Type: application/json" -d '{"query": "mutation { updateMessage(id: 1, content: \"I am not Batman\") { success } }"}' http://localhost:8080/graphql
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
  deleteMessage(id: 2) {
    success
    message
  }
}
```

```shell
curl -X POST -H "Content-Type: application/json" -d '{"query": "mutation { deleteMessage(id: 2) { success message } }"}' http://localhost:8080/graphql
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
  getAllMessagesForUser(userId:2) {
    id
    userId
    createdAt
    updatedAt
    content
  }
}
```
```shell
curl -X POST -H "Content-Type: application/json" -d '{"query": "query { getAllMessagesForUser(userId: 2) { id userId createdAt updatedAt content } }"}' http://localhost:8080/graphql
```

**Sample Response**

Success
```json
{
  "data": {
    "getAllMessagesForUser": [
      {
        "id": "4",
        "userId": "2",
        "createdAt": "2024-04-02T12:18:27.620364+00:00",
        "updatedAt": "2024-04-02T12:18:27.620364+00:00",
        "content": "Message 1 from User 2",
        "parentId": null,
        "user": {
          "name": ""
        }
      },
      {
        "id": "5",
        "userId": "2",
        "createdAt": "2024-04-02T12:18:27.636035+00:00",
        "updatedAt": "2024-04-02T12:18:27.636035+00:00",
        "content": "Message 2 from User 2",
        "parentId": null,
        "user": {
          "name": ""
        }
      },
      {
        "id": "6",
        "userId": "2",
        "createdAt": "2024-04-02T12:18:27.650145+00:00",
        "updatedAt": "2024-04-02T12:18:27.650145+00:00",
        "content": "Message 3 from User 2",
        "parentId": null,
        "user": {
          "name": ""
        }
      },
      {
        "id": "37",
        "userId": "2",
        "createdAt": "2024-04-02T12:21:36.212078+00:00",
        "updatedAt": "2024-04-02T12:21:36.212078+00:00",
        "content": "Message 1 from User 2",
        "parentId": null,
        "user": {
          "name": ""
        }
      },
      {
        "id": "38",
        "userId": "2",
        "createdAt": "2024-04-02T12:21:36.230136+00:00",
        "updatedAt": "2024-04-02T12:21:36.230136+00:00",
        "content": "Message 2 from User 2",
        "parentId": null,
        "user": {
          "name": ""
        }
      },
      {
        "id": "39",
        "userId": "2",
        "createdAt": "2024-04-02T12:21:36.246328+00:00",
        "updatedAt": "2024-04-02T12:21:36.246328+00:00",
        "content": "Message 3 from User 2",
        "parentId": null,
        "user": {
          "name": ""
        }
      },
      {
        "id": "70",
        "userId": "2",
        "createdAt": "2024-04-02T12:23:17.196612+00:00",
        "updatedAt": "2024-04-02T12:23:17.196612+00:00",
        "content": "Message 1 from User 2",
        "parentId": null,
        "user": {
          "name": ""
        }
      },
      {
        "id": "71",
        "userId": "2",
        "createdAt": "2024-04-02T12:23:17.216956+00:00",
        "updatedAt": "2024-04-02T12:23:17.216956+00:00",
        "content": "Message 2 from User 2",
        "parentId": null,
        "user": {
          "name": ""
        }
      },
      {
        "id": "72",
        "userId": "2",
        "createdAt": "2024-04-02T12:23:17.231876+00:00",
        "updatedAt": "2024-04-02T12:23:17.231876+00:00",
        "content": "Message 3 from User 2",
        "parentId": null,
        "user": {
          "name": ""
        }
      }
    ]
  }
}
```

- getMessagesByTimeRange
```graphql
query {
  getMessagesInTimeRangeForUser(userId: 3, start: "2023-01-01T00:00:00Z", end: "2025-01-02T00:00:00Z"){
 		content
    createdAt
  }
}
```
```shell
curl -X POST -H "Content-Type: application/json" -d '{"query": "query { getMessagesInTimeRangeForUser(userId: 3, start: \"2023-01-01T00:00:00Z\", end: \"2025-01-02T00:00:00Z\") { content createdAt } }"}' http://localhost:8080/graphql
```

**Sample Response**

Success
```json
{
  "data": {
    "getMessagesInTimeRangeForUser": [
      {
        "content": "Message 1 from User 3",
        "createdAt": "2024-04-02T12:18:27.670406+00:00",
        "id": "7",
        "user": {
          "name": ""
        }
      },
      {
        "content": "Message 2 from User 3",
        "createdAt": "2024-04-02T12:18:27.683630+00:00",
        "id": "8",
        "user": {
          "name": ""
        }
      },
      {
        "content": "Message 3 from User 3",
        "createdAt": "2024-04-02T12:18:27.695869+00:00",
        "id": "9",
        "user": {
          "name": ""
        }
      },
      {
        "content": "Message 1 from User 3",
        "createdAt": "2024-04-02T12:21:36.258290+00:00",
        "id": "40",
        "user": {
          "name": ""
        }
      },
      {
        "content": "Message 2 from User 3",
        "createdAt": "2024-04-02T12:21:36.272712+00:00",
        "id": "41",
        "user": {
          "name": ""
        }
      },
      {
        "content": "Message 3 from User 3",
        "createdAt": "2024-04-02T12:21:36.291681+00:00",
        "id": "42",
        "user": {
          "name": ""
        }
      },
      {
        "content": "Message 1 from User 3",
        "createdAt": "2024-04-02T12:23:17.244629+00:00",
        "id": "73",
        "user": {
          "name": ""
        }
      },
      {
        "content": "Message 2 from User 3",
        "createdAt": "2024-04-02T12:23:17.256670+00:00",
        "id": "74",
        "user": {
          "name": ""
        }
      },
      {
        "content": "Message 3 from User 3",
        "createdAt": "2024-04-02T12:23:17.268240+00:00",
        "id": "75",
        "user": {
          "name": ""
        }
      }
    ]
  }
}
```

getMessageThread
```graphql
query {
  getMessageThread(messageId:6) {
    id
    content
    updatedAt
    createdAt
    parentId
    user {
      name
    }
  }
}
```

```shell
curl -X POST -H "Content-Type: application/json" \
     -d '{"query": "query { getMessageThread(messageId: 6) { id content updatedAt createdAt parentId user { name } } }"}' \
      http://localhost:8080/graphql
```

**Sample Response**

Success
```json
{
  "data": {
    "getMessageThread": [
      {
        "id": "6",
        "content": "A",
        "updatedAt": "2024-04-02T08:30:21.852426+00:00",
        "createdAt": "2024-04-02T08:30:21.852426+00:00",
        "parentId": null,
        "user": {
          "name": "Carl"
        }
      },
      {
        "id": "7",
        "content": "A",
        "updatedAt": "2024-04-02T08:30:45.155447+00:00",
        "createdAt": "2024-04-02T08:30:45.155447+00:00",
        "parentId": 6,
        "user": {
          "name": "Carl"
        }
      },
      {
        "id": "8",
        "content": "B",
        "updatedAt": "2024-04-02T08:30:52.813426+00:00",
        "createdAt": "2024-04-02T08:30:52.813426+00:00",
        "parentId": 7,
        "user": {
          "name": "Carl"
        }
      },
      {
        "id": "11",
        "content": "D",
        "updatedAt": "2024-04-02T08:31:31.870908+00:00",
        "createdAt": "2024-04-02T08:31:31.870908+00:00",
        "parentId": 8,
        "user": {
          "name": "Carl"
        }
      },
      {
        "id": "12",
        "content": "XYZ",
        "updatedAt": "2024-04-02T09:11:06.281594+00:00",
        "createdAt": "2024-04-02T09:11:06.281594+00:00",
        "parentId": 11,
        "user": {
          "name": "Dominik"
        }
      }
    ]
  }
}
```


## Checklist
- [X] CI/CD (GitHub Actions)
- [X] Database (sqlx)
  - [X] User entity
  - [X] Message entity
  - [X] Create tables
  - [X] Query for User
  - [X] Query for Message
  - [X] Test
- [X] GraphQL (async-graphql)
  - [X] Endpoint
  - [X] Schema
  - [X] Query
  - [X] Mutation
  - [X] Test
- [X] CRUD for Message
- [X] Query for Message by user and time range
- [X] Advanced: Reply to Message
- [ ] Advanced: User registration/login


## Potential Improvements
- Better testing
- Better error handling / logging messages
- Authentication and Authorization
