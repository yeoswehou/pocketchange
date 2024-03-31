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

## Run Datbaase for dev
```
docker-compose up -d db
```

## How to run


## How to test

## Checklist
- [ ] Database (sqlx)
  - [X] User entity
  - [X] Message entity
  - [X] Create tables
  - [] Query for User
  - [] Query for Message
- [ ] GraphQL (async-graphql)
  - Endpoint
  - Schema
  - Query
  - Mutation
- [ ] CRUD for Message
- [ ] Query for Message by user and time range
- [ ] Advanced: Reply to Message
- [ ] Advanced: User registration/login
