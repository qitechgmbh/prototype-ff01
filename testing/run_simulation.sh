#!/bin/bash

cd "$(dirname "$0")"

DB_PATH="../testing/sandbox/data.db" 
SOCKET_PATH="/tmp/telemetry.sock" 
LIVE_PORT="9001" 
QUERY_PORT="9000"

ENV_EXPORT="export DB_PATH='$DB_PATH' SOCKET_PATH='$SOCKET_PATH' LIVE_PORT='$LIVE_PORT' QUERY_PORT='$QUERY_PORT';"

tmux kill-session -t telemetry_simulation 2>/dev/null
tmux new-session -d -s telemetry_simulation

tmux split-window -v
tmux split-window -h

tmux select-pane -t 0
tmux split-window -h

tmux send-keys -t 0 "$ENV_EXPORT cd ../telemetry-server; cargo run" C-m
tmux send-keys -t 1 "$ENV_EXPORT sleep 2.5; cd ../simulation; cargo run --example machine" C-m
tmux send-keys -t 2 "$ENV_EXPORT sleep 2.5; cd ../simulation; cargo run --example client_live" C-m
tmux send-keys -t 3 "$ENV_EXPORT sleep 2.5; cd ../simulation; cargo run --example client_query" C-m

tmux attach -t telemetry_simulation