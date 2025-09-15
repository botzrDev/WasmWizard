#!/bin/zsh

# Start Docker Compose for dev environment
cd wasm-wizard
docker-compose -f docker-compose.dev.yml up -d

# Wait for the database to be healthy
DB_HEALTH=1
for i in {1..20}; do
  STATUS=$(docker inspect --format='{{.State.Health.Status}}' wasm-wizard_dev_db 2>/dev/null)
  if [[ "$STATUS" == "healthy" ]]; then
    DB_HEALTH=0
    break
  fi
  echo "Waiting for database to be healthy... ($i)"
  sleep 2
done
if [[ $DB_HEALTH -ne 0 ]]; then
  echo "Database did not become healthy in time. Exiting."
  exit 1
fi

# Start the backend
cargo run &
BACKEND_PID=$!

# Save backend PID for stopping later
echo $BACKEND_PID > ../pids.txt

# Notify user
echo "Application started. Backend PID: $BACKEND_PID"
