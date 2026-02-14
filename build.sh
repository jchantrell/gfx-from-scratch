#!/usr/bin/env bash
set -euo pipefail

wasm-pack build --target web --out-dir pkg
python3 -m http.server
