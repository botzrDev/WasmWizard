#!/bin/bash
# Secret generation script for WasmWiz production deployment
# This script generates secure random secrets for production use

set -euo pipefail

SECRETS_DIR="$(dirname "$0")/../secrets"
mkdir -p "$SECRETS_DIR"

echo "Generating production secrets for WasmWiz..."

# Generate database password (32 characters)
echo "Generating database password..."
openssl rand -base64 32 > "$SECRETS_DIR/db_password.txt"
DB_PASSWORD="$(tr -d '\n' < "$SECRETS_DIR/db_password.txt")"

# Generate database URL secret for application containers
echo "Creating database URL secret..."
printf "postgres://wasmwiz:%s@postgres:5432/wasmwiz\n" "$DB_PASSWORD" > \
  "$SECRETS_DIR/database_url.txt"

# Generate API salt (48 characters for extra security)
echo "Generating API salt..."
openssl rand -base64 48 > "$SECRETS_DIR/api_salt.txt"

# Generate Grafana admin password (32 characters)
echo "Generating Grafana admin password..."
openssl rand -base64 32 > "$SECRETS_DIR/grafana_password.txt"

# Set secure permissions
chmod 600 "$SECRETS_DIR"/*.txt

echo "✅ Secrets generated successfully in $SECRETS_DIR"
echo "⚠️  Make sure to:"
echo "   1. Back up these secrets securely"
echo "   2. Never commit them to version control"
echo "   3. Use proper file permissions (600)"

echo ""
echo "Generated files:"
ls -la "$SECRETS_DIR"/*.txt
