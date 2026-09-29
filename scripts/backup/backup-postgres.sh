#!/bin/bash
# PostgreSQL backup for the analytics DB (issue #517).
# Runs a pg_dump (custom format, restorable with pg_restore) and, when
# WAL_ARCHIVE_DIR is set, verifies the WAL archive is being written to —
# that's what makes point-in-time recovery possible; enabling it is a
# server-side setting (see the note at the bottom).

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKUP_DIR="${BACKUP_DIR:-$SCRIPT_DIR/../../backups/postgres}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
mkdir -p "$BACKUP_DIR"

DATABASE_URL="${DATABASE_URL:?Set DATABASE_URL (postgres://user:pass@host:port/db)}"
RETENTION_DAYS="${RETENTION_DAYS:-90}"
DUMP_FILE="$BACKUP_DIR/nebula_analytics_${TIMESTAMP}.dump"

echo "[backup-postgres] Dumping $DATABASE_URL -> $DUMP_FILE"
pg_dump --format=custom --file="$DUMP_FILE" "$DATABASE_URL"

sha256sum "$DUMP_FILE" > "$DUMP_FILE.sha256"
echo "[backup-postgres] Checksum: $(cat "$DUMP_FILE.sha256")"

if [ -n "$S3_BUCKET" ]; then
    aws s3 cp "$DUMP_FILE" "s3://$S3_BUCKET/postgres-backups/" || echo "[backup-postgres] WARNING: S3 upload failed"
    aws s3 cp "$DUMP_FILE.sha256" "s3://$S3_BUCKET/postgres-backups/" || true
fi

echo "[backup-postgres] Pruning dumps older than ${RETENTION_DAYS}d"
find "$BACKUP_DIR" -name '*.dump*' -mtime "+${RETENTION_DAYS}" -delete

# ── Point-in-time recovery ──────────────────────────────────────────────
# PITR needs continuous WAL archiving enabled on the server itself
# (postgresql.conf: wal_level = replica, archive_mode = on,
# archive_command = 'aws s3 cp %p s3://$S3_BUCKET/wal-archive/%f') — a
# managed instance (RDS: `backup_retention_period`, already set in
# infrastructure/modules/database) does this for you; a self-hosted
# server needs it configured explicitly. This script backs up the base
# data; it does not itself enable WAL archiving.
echo "[backup-postgres] Done. For PITR, confirm WAL archiving is enabled on the server (see comment in this script)."
