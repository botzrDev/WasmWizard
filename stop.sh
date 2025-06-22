#!/bin/zsh

# Stop the application using saved PIDs
if [ -f pids.txt ]; then
  while read PID; do
    kill $PID
  done < pids.txt
  rm pids.txt
  echo "Application stopped."
else
  echo "No running application found."
fi
