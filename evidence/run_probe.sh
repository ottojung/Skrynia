#!/bin/bash
# usage: run_probe.sh <binary> <root.wasm> <port> <probe.py> <tag>
set -e
BIN="$1"; ROOT="$2"; PORT="$3"; PROBE="$4"; TAG="$5"
DIR="/workspace/skrynia-http-hardening-voCxtXBP-20261121/evidence/runtime"
mkdir -p "$DIR"
OP="$DIR/op-$TAG"; US="$DIR/us-$TAG"
rm -rf "$OP" "$US"; mkdir -p "$OP" "$US"
SKRYNIA_ROOT_WASM="$ROOT" SKRYNIA_OPERATOR_DIR="$OP" SKRYNIA_USER_DIR="$US" \
SKRYNIA_BIND="127.0.0.1:$PORT" "$BIN" >"$DIR/server-$TAG.log" 2>&1 &
PID=$!
for i in $(seq 1 100); do
  if python3 -c "import socket,sys; s=socket.create_connection(('127.0.0.1',$PORT),0.2); s.close()" 2>/dev/null; then break; fi
  sleep 0.1
done
python3 "$PROBE" "$PORT" | tee "$DIR/$TAG-result.txt"
kill -TERM $PID 2>/dev/null || true
wait $PID 2>/dev/null || true
