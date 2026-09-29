#!/bin/bash
# Restore a PostgreSQL dump made by backup-postgres.sh (issue #517).
# Usage: ./restore-postgres.sh --dump <file> [--verify-only] [--target-db <url>]

set -e

DUMP_FILE=""
TARGET_DB="${DATABASE_URL:-}"
VERIFY_ONLY=false

while [ $# -gt 0 ]; do
    case "$1" in
        --dump) DUMP_FILE="$2"; shift 2 ;;
        --target-db) TARGET_DB="$2"; shift 2 ;;
        --verify-only) VERIFY_ONLY=true; shift ;;
        *) echo "Unknown argument: $1"; exit 1 ;;
    esac
done

[ -n "$DUMP_FILE" ] || { echo "Usage: $0 --dump <file> [--target-db <url>] [--verify-only]"; exit 1; }
[ -f "$DUMP_FILE" ] || { echo "No such file: $DUMP_FILE"; exit 1; }

if [ -f "$DUMP_FILE.sha256" ]; then
    echo "[restore-postgres] Verifying checksum..."
    sha256sum -c "$DUMP_FILE.sha256"
else
    echo "[restore-postgres] WARNING: no checksum file found for $DUMP_FILE"
fi

if [ "$VERIFY_ONLY" = true ]; then
    echo "[restore-postgres] --verify-only: checksum OK, not restoring. Done."
    exit 0
fi

[ -n "$TARGET_DB" ] || { echo "Set --target-db or DATABASE_URL to restore into"; exit 1; }

echo "[restore-postgres] Restoring $DUMP_FILE -> $TARGET_DB"
pg_restore --clean --if-exists --no-owner --dbname="$TARGET_DB" "$DUMP_FILE"
echo "[restore-postgres] Restore complete."
