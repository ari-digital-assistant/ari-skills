---
name: wasm-echo
description: Minuscola skill WASM di test. Restituisce un saluto fisso dall'interno del suo modulo sandboxed. Usare solo per testare il loader WASM.
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
        - keywords: [wasm, eco]
          weight: 0.95
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# WASM Echo (Italiano)

Skill di riferimento che esiste unicamente per verificare che il loader WASM funzioni end-to-end. Il modulo è un frammento WAT scritto a mano compilato in WebAssembly. Restituisce la stringa letterale "wasm hello" dalla propria memoria lineare.
