---
name: wasm-echo
description: Tiny WASM smoke-test skill. Returns a fixed greeting from inside its sandboxed module. Use only for testing the WASM loader.
license: MIT
metadata:
  ari:
    id: dev.heyari.wasmecho
    version: "0.1.0"
    author: Ari core team
    engine: ">=0.1"
    capabilities: []
    languages: [en, it]
    specificity: high
    matching:
      patterns:
        - keywords: [wasm, echo]
          weight: 0.95
    examples:
      - text: "wasm echo"
      - text: "echo test"
      - text: "test the wasm loader"
      - text: "run the echo skill"
      - text: "wasm hello"
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# WASM Echo

Reference skill that exists purely to prove the WASM loader works end to end.
A minimal Rust SDK module (`src/lib.rs`, built via `build.sh`) exporting the
ABI v1 surface (`memory`, `ari_alloc`, `score`, `execute`). It returns the
`greeting` string resolved per-locale from `strings/{locale}.json` via
`ari::t()` — the canonical example of WASM-skill output localization
("wasm hello" in English, "ciao da wasm" in Italian).

## Example utterance

- "wasm echo"
