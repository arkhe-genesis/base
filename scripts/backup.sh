#!/bin/bash
# scripts/backup.sh
# Automated backup script for Cathedral ARKHE (PostgreSQL & Redis)
#
# Selo: CATHEDRAL-ARKHE-8000-BACKUP-v2.1.0-2026-06-19

set -e

BACKUP_DIR="/mnt/persist/backups"
DATE=$(date +"%Y%m%d_%H%M%S")
PG_CONTAINER="cathedral-postgres"
REDIS_CONTAINER="cathedral-redis"
RETENTION_DAYS=7

mkdir -p "${BACKUP_DIR}/postgres"
mkdir -p "${BACKUP_DIR}/redis"

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

log "Starting backup process..."

# Backup PostgreSQL
log "Backing up PostgreSQL..."
PG_BACKUP_FILE="${BACKUP_DIR}/postgres/cathedral_db_${DATE}.sql.gz"
if docker ps | grep -q "${PG_CONTAINER}"; then
    docker exec "${PG_CONTAINER}" pg_dump -U cathedral cathedral | gzip > "${PG_BACKUP_FILE}"
    log "PostgreSQL backup created: ${PG_BACKUP_FILE}"
else
    log "ERROR: PostgreSQL container '${PG_CONTAINER}' is not running."
    exit 1
fi

# Backup Redis (Trigger BGSAVE and copy dump.rdb)
log "Backing up Redis..."
REDIS_BACKUP_FILE="${BACKUP_DIR}/redis/cathedral_redis_${DATE}.rdb"
if docker ps | grep -q "${REDIS_CONTAINER}"; then
    # Trigger a background save
    docker exec "${REDIS_CONTAINER}" redis-cli BGSAVE

    # Wait for the save to complete (basic approach)
    log "Waiting for Redis BGSAVE to complete..."
    sleep 5

    # Copy the rdb file out of the container
    # Adjust path if Redis inside container saves somewhere else
    docker cp "${REDIS_CONTAINER}:/data/dump.rdb" "${REDIS_BACKUP_FILE}" || log "Warning: Failed to copy dump.rdb, maybe save takes longer or path is wrong."
    log "Redis backup created: ${REDIS_BACKUP_FILE}"
else
    log "ERROR: Redis container '${REDIS_CONTAINER}' is not running."
    exit 1
fi

# Cleanup old backups
log "Cleaning up backups older than ${RETENTION_DAYS} days..."
find "${BACKUP_DIR}/postgres" -type f -name "*.sql.gz" -mtime +${RETENTION_DAYS} -exec rm {} \;
find "${BACKUP_DIR}/redis" -type f -name "*.rdb" -mtime +${RETENTION_DAYS} -exec rm {} \;

log "Backup process completed successfully."
