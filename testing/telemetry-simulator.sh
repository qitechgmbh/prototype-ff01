#!/usr/bin/env bash

# Get the directory of this script
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR" || exit 1

cargo run --manifest-path telemetry-server/Cargo.toml &
PID1=$!

cargo run --manifest-path telemetry.journal/Cargo.toml &
PID2=$!

wait $PID1 $PID2