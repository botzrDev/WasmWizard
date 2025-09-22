#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/wasmwiz"
PIDS_FILE="$SCRIPT_DIR/pids.txt"

# Stop the application using saved PIDs
if [ -f "$PIDS_FILE" ]; then
  while IFS= read -r PID; do
    if [[ -n "$PID" ]]; then
      kill "$PID" 2>/dev/null || true
    fi
  done < "$PIDS_FILE"
  rm "$PIDS_FILE"
  echo "Backend application stopped (by PID)."
else
  echo "No running backend application found by PID file."
fi

# Also kill any remaining wasm-wizard processes (except this script)
PIDS=$(ps aux | grep wasm-wizard | grep -v grep | grep -v stop.sh | awk '{print $2}' || true)
if [ -n "$PIDS" ]; then
  echo "Killing remaining wasm-wizard processes: $PIDS"
  for PID in $PIDS; do
    kill -9 $PID 2>/dev/null
  done
fi

# Stop Docker Compose services
cd "$PROJECT_DIR"
docker-compose -f docker-compose.dev.yml down

echo "Docker Compose services stopped."
