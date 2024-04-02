#!/bin/sh

# Construct the DATABASE_URL
export DATABASE_URL="postgresql://$POSTGRES_USER:$POSTGRES_PASSWORD@$DB_HOST/$POSTGRES_DB"

echo "Starting the backend server..."
/app/backend

sleep 5

echo "Populating the database..."
/usr/local/bin/populate.sh
