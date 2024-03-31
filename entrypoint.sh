#!/bin/sh

# Construct the DATABASE_URL
export DATABASE_URL="postgresql://$POSTGRES_USER:$POSTGRES_PASSWORD@$DB_HOST/$POSTGRES_DB"

exec "$@"
