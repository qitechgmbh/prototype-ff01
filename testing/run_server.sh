#!/bin/bash

cd "$(dirname "$0")"

DB_PATH="../testing/sandbox/data.db" 
SOCKET_PATH="/tmp/telemetry.sock" 
LIVE_PORT="9001" 
QUERY_PORT="9000"

export QITECH_TELEMETRY_DB_PATH="$DB_PATH"
export QITECH_TELEMETRY_SOCKET_PATH="$SOCKET_PATH"
export QITECH_TELEMETRY_LIVE_PORT="$LIVE_PORT"
export QITECH_TELEMETRY_QUERY_PORT="$QUERY_PORT"

cd ../telemetry-server
cargo run