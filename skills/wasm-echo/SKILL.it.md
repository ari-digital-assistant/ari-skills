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
    # NB: "eco" da solo non basta — il pattern richiede entrambe le
    # parole. Le frasi senza "wasm" che iniziano con un verbo tipo
    # "esegui"/"avvia" verrebbero rivendicate dalla skill built-in
    # `open`, quindi qui restano fuori: il router non le vedrebbe mai.
    examples:
      - text: "eco wasm"
      - text: "prova l'eco wasm"
      - text: "prova il loader wasm"
      - text: "esegui la skill eco wasm"
      - text: "saluto dal modulo wasm"
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# WASM Echo (Italiano)

Skill di riferimento che esiste unicamente per verificare che il loader WASM funzioni end-to-end. Un modulo Rust SDK minimale (`src/lib.rs`, compilato con `build.sh`) che esporta la superficie ABI v1 (`memory`, `ari_alloc`, `score`, `execute`). Restituisce la stringa `greeting` risolta per lingua da `strings/{locale}.json` tramite `ari::t()` — l'esempio canonico di localizzazione dell'output di una skill WASM ("wasm hello" in inglese, "ciao da wasm" in italiano).
