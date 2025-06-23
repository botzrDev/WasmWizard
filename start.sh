#!/bin/zsh

# Start the backend
cd wasmwiz
cargo run &
BACKEND_PID=$!

# Start the frontend (if applicable, replace with actual frontend start command)
cd static
# Example: npm start or any other command
# Uncomment and replace the following line with the actual command
# npm start &
# FRONTEND_PID=$!

# Save PIDs to a file for stopping later
echo $BACKEND_PID > ../pids.txt
# echo $FRONTEND_PID >> ../pids.txt

# Notify user
echo "Application started. Backend PID: $BACKEND_PID"
