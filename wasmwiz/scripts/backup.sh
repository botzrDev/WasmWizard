#!/bin/bash
# Wasm Wizard Production Backup Script
# This script creates backups of the PostgreSQL database and important files

set -euo pipefail

# Configuration
BACKUP_DIR="${BACKUP_DIR:-/opt/wasm-wizard/backups}"
RETENTION_DAYS="${RETENTION_DAYS:-7}"
POSTGRES_CONTAINER="${POSTGRES_CONTAINER:-wasm-wizard_postgres_1}"
DATABASE_NAME="${DATABASE_NAME:-wasm-wizard}"
DATABASE_USER="${DATABASE_USER:-wasm-wizard}"

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Function to log messages
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

# Function to create database backup
backup_database() {
    local backup_file="$BACKUP_DIR/db_backup_$(date +%Y%m%d_%H%M%S).sql.gz"
    
    log "Creating database backup: $backup_file"
    
    docker exec "$POSTGRES_CONTAINER" pg_dump \
        -U "$DATABASE_USER" \
        -d "$DATABASE_NAME" \
        --no-password \
        --clean \
        --if-exists \
        --create \
        --verbose \
        | gzip > "$backup_file"
    
    if [[ -f "$backup_file" && -s "$backup_file" ]]; then
        log "Database backup completed successfully: $backup_file"
        echo "$backup_file"
    else
        log "ERROR: Database backup failed or is empty"
        exit 1
    fi
}

# Function to backup application configuration
backup_config() {
    local config_backup="$BACKUP_DIR/config_backup_$(date +%Y%m%d_%H%M%S).tar.gz"
    
    log "Creating configuration backup: $config_backup"
    
    tar -czf "$config_backup" \
        -C /opt/wasm-wizard \
        docker-compose.yml \
        docker-compose.production.yml \
        .env \
        secrets/ \
        monitoring/ \
        k8s/ \
        2>/dev/null || true
    
    if [[ -f "$config_backup" ]]; then
        log "Configuration backup completed: $config_backup"
        echo "$config_backup"
    else
        log "WARNING: Configuration backup failed"
    fi
}

# Function to verify backup integrity
verify_backup() {
    local backup_file="$1"
    
    log "Verifying backup integrity: $backup_file"
    
    if [[ "$backup_file" == *.sql.gz ]]; then
        # Verify SQL backup by checking if it can be decompressed and contains SQL
        if zcat "$backup_file" | head -n 50 | grep -q "PostgreSQL database dump"; then
            log "SQL backup verification passed"
            return 0
        else
            log "ERROR: SQL backup verification failed"
            return 1
        fi
    elif [[ "$backup_file" == *.tar.gz ]]; then
        # Verify tar backup
        if tar -tzf "$backup_file" >/dev/null 2>&1; then
            log "Configuration backup verification passed"
            return 0
        else
            log "ERROR: Configuration backup verification failed"
            return 1
        fi
    fi
}

# Function to clean old backups
cleanup_old_backups() {
    log "Cleaning up backups older than $RETENTION_DAYS days"
    
    find "$BACKUP_DIR" -name "*.sql.gz" -mtime +"$RETENTION_DAYS" -delete
    find "$BACKUP_DIR" -name "*.tar.gz" -mtime +"$RETENTION_DAYS" -delete
    
    log "Cleanup completed"
}

# Function to send backup notification
send_notification() {
    local status="$1"
    local message="$2"
    
    # Add your notification logic here (Slack, email, etc.)
    log "NOTIFICATION [$status]: $message"
    
    # Example: Send to a webhook
    if [[ -n "${WEBHOOK_URL:-}" ]]; then
        curl -X POST "$WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -d "{\"text\": \"Wasm Wizard Backup $status: $message\"}" \
            >/dev/null 2>&1 || true
    fi
}

# Main backup function
main() {
    local start_time=$(date +%s)
    
    log "Starting Wasm Wizard backup process"
    
    # Check if PostgreSQL container is running
    if ! docker ps | grep -q "$POSTGRES_CONTAINER"; then
        log "ERROR: PostgreSQL container '$POSTGRES_CONTAINER' is not running"
        send_notification "FAILED" "PostgreSQL container not running"
        exit 1
    fi
    
    # Create backups
    local db_backup
    local config_backup
    
    db_backup=$(backup_database)
    config_backup=$(backup_config)
    
    # Verify backups
    if verify_backup "$db_backup"; then
        log "Database backup verified successfully"
    else
        log "ERROR: Database backup verification failed"
        send_notification "FAILED" "Database backup verification failed"
        exit 1
    fi
    
    if [[ -n "$config_backup" ]] && verify_backup "$config_backup"; then
        log "Configuration backup verified successfully"
    fi
    
    # Cleanup old backups
    cleanup_old_backups
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log "Backup process completed successfully in ${duration}s"
    send_notification "SUCCESS" "Backup completed in ${duration}s. Database: $(basename "$db_backup")"
}

# Error handling
trap 'log "ERROR: Backup process failed"; send_notification "FAILED" "Backup process encountered an error"' ERR

# Run main function
main "$@"