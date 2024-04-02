#!/bin/bash

# GraphQL endpoint
GRAPHQL_ENDPOINT="http://localhost:8080/graphql"

# Create users
for i in {1..5}; do
  curl -s -X POST "$GRAPHQL_ENDPOINT" -H "Content-Type: application/json" \
       -d "{\"query\": \"mutation { createUser(name: \\\"User $i\\\") { success message } }\"}" 
done

# Create messages for each user
for userId in {1..5}; do
  for i in {1..3}; do
    curl -s -X POST "$GRAPHQL_ENDPOINT" -H "Content-Type: application/json" \
         -d "{\"query\": \"mutation { createMessage(userId: $userId, content: \\\"Message $i from User $userId\\\") { success message } }\"}" 
  done
done

# Create a thread by setting parentId for some messages
curl -s -X POST "$GRAPHQL_ENDPOINT" -H "Content-Type: application/json" \
     -d "{\"query\": \"mutation { createMessage(userId: 1, content: \\\"Reply to Message 1 from User 1\\\", parentId: 1) { success message } }\"}" 

# Fetch all users
echo "Fetching all users:"
curl -s -X POST "$GRAPHQL_ENDPOINT" -H "Content-Type: application/json" \
     -d "{\"query\": \"query { getAllUsers { id name } }\"}" 

# Fetch all messages for a user
echo "Fetching all messages for User 1:"
curl -s -X POST "$GRAPHQL_ENDPOINT" -H "Content-Type: application/json" \
     -d "{\"query\": \"query { getAllMessagesForUser(userId: 1) { id content createdAt } }\"}" 

# Fetch message thread
echo "Fetching message thread for Message 1:"
curl -s -X POST "$GRAPHQL_ENDPOINT" -H "Content-Type: application/json" \
     -d "{\"query\": \"query { getMessageThread(messageId: 1) { id content parentId } }\"}" 
