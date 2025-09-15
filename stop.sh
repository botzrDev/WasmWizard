#!/bin/zsh

# Stop the application using saved PIDs
if [ -f pids.txt ]; then
  while read PID; do
    kill $PID 2>/dev/null
  done < pids.txt
  rm pids.txt
  echo "Backend application stopped (by PID)."
else
  echo "No running backend application found by PID file."
fi

# Also kill any remaining wasm-wizard processes (except this script)
PIDS=$(ps aux | grep wasm-wizard | grep -v grep | grep -v stop.sh | awk '{print $2}')
if [ -n "$PIDS" ]; then
  echo "Killing remaining wasm-wizard processes: $PIDS"
  for PID in $PIDS; do
    kill -9 $PID 2>/dev/null
  done
fi

# Stop Docker Compose services
cd wasm-wizard
docker-compose -f docker-compose.dev.yml down
cd ..

echo "Docker Compose services stopped."
