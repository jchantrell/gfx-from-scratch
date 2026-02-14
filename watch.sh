#!/usr/bin/env bash
set -euo pipefail

VITE_PID=""

cleanup() {
    if [[ -n "$VITE_PID" ]]; then
        kill "$VITE_PID" 2>/dev/null
        wait "$VITE_PID" 2>/dev/null
    fi
    exit 0
}

trap cleanup INT TERM

build() {
    if ! wasm-pack build --target web --out-dir pkg 2>&1; then
        echo "--- build failed ---"
    fi
}

build

npx vite &
VITE_PID=$!

while inotifywait -r -e modify,create,delete,move src/ Cargo.toml 2>/dev/null; do
    build
done
