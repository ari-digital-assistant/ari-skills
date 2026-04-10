#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/my_ari_skill.wasm skill.wasm
echo "wrote skill.wasm ($(stat -c %s skill.wasm) bytes)"
