#!/bin/bash
# Wasm Wizard Production Restore Script
# This script restores the PostgreSQL database from a backup

set -euo pipefail

# Configuration
BACKUP_DIR="${BACKUP_DIR:-/opt/wasm-wizard/backups}"
POSTGRES_CONTAINER="${POSTGRES_CONTAINER:-wasm-wizard_postgres_1}"
DATABASE_NAME="${DATABASE_NAME:-wasm-wizard}"
DATABASE_USER="${DATABASE_USER:-wasm-wizard}"

# Function to log messages
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

# Function to list available backups
list_backups() {
    log "Available database backups:"
    ls -la "$BACKUP_DIR"/db_backup_*.sql.gz 2>/dev/null | sort -r || {
        log "No backups found in $BACKUP_DIR"
        exit 1
    }
}

# Function to restore database
restore_database() {
    local backup_file="$1"
    
    if [[ ! -f "$backup_file" ]]; then
        log "ERROR: Backup file not found: $backup_file"
        exit 1
    fi
    
    log "Restoring database from: $backup_file"
    
    # Verify backup file integrity
    if ! zcat "$backup_file" | head -n 10 | grep -q "PostgreSQL database dump"; then
        log "ERROR: Backup file appears to be corrupted"
        exit 1
    fi
    
    # Stop the Wasm Wizard application to prevent database connections
    log "Stopping Wasm Wizard application..."
    docker-compose stop wasm-wizard || true
    
    # Wait for connections to close
    sleep 5
    
    # Drop existing connections (if any)
    docker exec "$POSTGRES_CONTAINER" psql -U "$DATABASE_USER" -d postgres -c \
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '$DATABASE_NAME' AND pid <> pg_backend_pid();" || true
    
    # Restore the database
    log "Restoring database..."
    zcat "$backup_file" | docker exec -i "$POSTGRES_CONTAINER" psql -U "$DATABASE_USER" -d postgres
    
    if [[ $? -eq 0 ]]; then
        log "Database restore completed successfully"
    else
        log "ERROR: Database restore failed"
        exit 1
    fi
    
    # Restart the Wasm Wizard application
    log "Starting Wasm Wizard application..."
    docker-compose start wasm-wizard
    
    # Wait for application to start
    sleep 10
    
    # Verify application is healthy
    if curl -f http://localhost:8080/health >/dev/null 2>&1; then
        log "Application is healthy after restore"
    else
        log "WARNING: Application health check failed after restore"
    fi
}

# Function to create a pre-restore backup
create_pre_restore_backup() {
    local pre_backup="$BACKUP_DIR/pre_restore_backup_$(date +%Y%m%d_%H%M%S).sql.gz"
    
    log "Creating pre-restore backup: $pre_backup"
    
    docker exec "$POSTGRES_CONTAINER" pg_dump \
        -U "$DATABASE_USER" \
        -d "$DATABASE_NAME" \
        --no-password \
        --clean \
        --if-exists \
        --create \
        | gzip > "$pre_backup"
    
    log "Pre-restore backup created: $pre_backup"
}

# Main function
main() {
    local backup_file="${1:-}"
    
    if [[ -z "$backup_file" ]]; then
        log "Usage: $0 <backup_file>"
        log "       $0 latest"
        echo
        list_backups
        exit 1
    fi
    
    # Handle 'latest' option
    if [[ "$backup_file" == "latest" ]]; then
        backup_file=$(ls -t "$BACKUP_DIR"/db_backup_*.sql.gz 2>/dev/null | head -n1)
        if [[ -z "$backup_file" ]]; then
            log "ERROR: No backup files found"
            exit 1
        fi
        log "Using latest backup: $backup_file"
    fi
    
    # Convert relative path to absolute
    if [[ ! "$backup_file" =~ ^/ ]]; then
        backup_file="$BACKUP_DIR/$backup_file"
    fi
    
    # Check if PostgreSQL container is running
    if ! docker ps | grep -q "$POSTGRES_CONTAINER"; then
        log "ERROR: PostgreSQL container '$POSTGRES_CONTAINER' is not running"
        exit 1
    fi
    
    # Confirmation prompt
    echo "WARNING: This will replace the current database with the backup!"
    echo "Backup file: $backup_file"
    echo "Database: $DATABASE_NAME"
    read -p "Are you sure you want to continue? (yes/no): " -r
    if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        log "Restore operation cancelled"
        exit 0
    fi
    
    # Create pre-restore backup
    create_pre_restore_backup
    
    # Perform restore
    restore_database "$backup_file"
    
    log "Restore operation completed successfully"
}

# Error handling
trap 'log "ERROR: Restore process failed"' ERR

# Run main function
main "$@"