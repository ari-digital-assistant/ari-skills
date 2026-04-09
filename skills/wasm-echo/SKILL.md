---
name: wasm-echo
description: Tiny WASM smoke-test skill. Returns a fixed greeting from inside its sandboxed module. Use only for testing the WASM loader.
license: MIT
metadata:
  ari:
    id: dev.heyari.wasmecho
    version: "0.1.0"
    author: Ari core team
    engine: ">=0.3,<0.4"
    capabilities: []
    languages: [en]
    specificity: high
    matching:
      patterns:
        - keywords: [wasm, echo]
          weight: 0.95
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# WASM Echo

Reference skill that exists purely to prove the WASM loader works end to end.
The module is a hand-written WAT fragment compiled to WebAssembly. It exports
the ABI v1 surface (`memory`, `ari_alloc`, `score`, `execute`) and returns
the literal string "wasm hello" from inside its linear memory.

## Example utterance

- "wasm echo"
