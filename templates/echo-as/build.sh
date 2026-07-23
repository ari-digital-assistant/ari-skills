#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

npm install --silent
npx asc assembly/index.ts --outFile skill.wasm --optimize --exportRuntime --use abort=
echo "wrote skill.wasm ($(stat -c %s skill.wasm) bytes)"
