#!/bin/bash
# deploy.sh - Deploy the ServerModule to SpacetimeDB
#
# This script builds and deploys the ServerModule to a SpacetimeDB instance.
# Usage: ./deploy.sh [database_name] [host]
#
# If database_name is not provided, it will use "unreal_game" as default.
# If host is not provided, it will use "localhost:3000" as default.

set -e

# Parse command line arguments
DB_NAME=${1:-unreal_game}
HOST=${2:-localhost:3000}

echo "========================================"
echo "SpacetimeDB UnrealClient Deployment Tool"
echo "========================================"
echo "Deploying ServerModule to:"
echo "Database: $DB_NAME"
echo "Host:     $HOST"
echo "========================================"

# Check if spacetime CLI is installed
if ! command -v spacetime &> /dev/null
then
    echo "Error: SpacetimeDB CLI not found!"
    echo "Please install it with: cargo install spacetime"
    exit 1
fi

# Build the CustomServerModule first (since ServerModule depends on it)
echo "Building CustomServerModule..."
cd ../CustomServerModule
cargo build --release

# Build the ServerModule
echo "Building ServerModule..."
cd ../ServerModule
cargo build --release

# Check if the database exists, if not create it
echo "Checking if database exists..."
if ! spacetime db list | grep -q "$DB_NAME"; then
    echo "Creating database $DB_NAME..."
    spacetime db create "$DB_NAME"
fi

# Deploy the module
echo "Deploying module to SpacetimeDB..."
spacetime db publish "$DB_NAME" --host "$HOST"

echo "========================================"
echo "Deployment completed successfully!"
echo "========================================"
echo "You can now connect to your database at:"
echo "$HOST/$DB_NAME"
echo "========================================" 