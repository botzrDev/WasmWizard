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

# Also kill any remaining wasmwiz processes (except this script)
PIDS=$(ps aux | grep wasmwiz | grep -v grep | grep -v stop.sh | awk '{print $2}')
if [ -n "$PIDS" ]; then
  echo "Killing remaining wasmwiz processes: $PIDS"
  kill -9 $PIDS
fi

# Stop Docker Compose services
cd wasmwiz
docker-compose -f docker-compose.dev.yml down
cd ..

echo "Docker Compose services stopped."
