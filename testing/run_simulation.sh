#!/bin/bash

ghostty -e bash -c '
tmux kill-session -t perfect_split 2>/dev/null

tmux new-session -d -s perfect_split

tmux split-window -v
# tmux split-window -v

tmux send-keys -t 0 "cd ../telemetry-server; cargo run" C-m
tmux send-keys -t 1 "sleep 1; cd ../machine-simulator; cargo run" C-m

tmux attach -t perfect_split
' &

# ghostty -e bash -c '
# python simulation/client.py
# '