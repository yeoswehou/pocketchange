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
## How to run


## How to test

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
