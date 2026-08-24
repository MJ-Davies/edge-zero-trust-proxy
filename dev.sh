#!/bin/bash

echo "===================================================="
echo "Starting Edge Zero Trust Proxy Environment..."
echo "===================================================="

mkdir -p ./log

echo "[Mock Backend] Starting background process..."
(cd mock-backend && npx wrangler dev --port 8788) > ./log/mock-backend.log 2>&1 &
BACKEND_PID=$!
echo "Mock Backend Root PID: $BACKEND_PID"

sleep 2

echo "[Rust Proxy] Starting background process..."
(cd edge-zero-trust-proxy && npx wrangler dev) > ./log/rust-proxy.log 2>&1 &
PROXY_PID=$!
echo "Rust Proxy Root PID: $PROXY_PID"

echo ""
echo "===================================================="
echo "Both services are running silently in the background."
echo "Output is being logged to the ./log/ folder."
echo "Type 'EXIT' and press Enter to kill all processes and delete the log folder."
echo "===================================================="

while true; do
    read -r -p "Enter command: " USER_INPUT
    if [[ "$USER_INPUT" == "EXIT" || "$USER_INPUT" == "exit" ]]; then
        break
    fi
done

echo ""
echo "Shutting down processes..."

kill_process_tree() {
    local parent_pid=$1
    if command -v pgrep >/dev/null 2>&1; then
        local children
        children=$(pgrep -P "$parent_pid")
        for child in $children; do
            kill_process_tree "$child"
        done
    fi
    kill -9 "$parent_pid" 2>/dev/null
}

if kill -0 "$BACKEND_PID" 2>/dev/null; then
    echo "Terminating Mock Backend process tree (PID: $BACKEND_PID)..."
    kill_process_tree "$BACKEND_PID"
fi

if kill -0 "$PROXY_PID" 2>/dev/null; then
    echo "Terminating Rust Proxy process tree (PID: $PROXY_PID)..."
    kill_process_tree "$PROXY_PID"
fi

echo "Cleaning up log folder..."
rm -rf ./log

echo "All services stopped and logs deleted cleanly."