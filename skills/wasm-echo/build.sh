#!/usr/bin/env bash
# Compile skill.wat to skill.wasm. Requires `wat2wasm` from wabt:
#   https://github.com/WebAssembly/wabt
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
wat2wasm skill.wat -o skill.wasm
echo "wrote skill.wasm ($(stat -c %s skill.wasm) bytes)"
