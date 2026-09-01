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
        - keywords: [wasm, echo]
          weight: 0.95
    # NB: "echo" da solo non basta — il pattern richiede entrambe le
    # parole. Le frasi senza "wasm" che iniziano con un verbo tipo
    # "esegui"/"avvia" verrebbero rivendicate dalla skill built-in
    # `open`, quindi qui restano fuori: il router non le vedrebbe mai.
    examples:
      - text: "esegui il loader wasm"
        weight: 0.95
      - text: "esegui il caricatore wasm"
        weight: 0.95
      - text: "esegui il modulo wasm"
        weight: 0.95
      - text: "esegui la skill di test wasm"
        weight: 0.95
      - text: "lancia il loader wasm"
        weight: 0.95
      - text: "lancia il caricatore wasm"
        weight: 0.95
      - text: "lancia il modulo wasm"
        weight: 0.95
      - text: "lancia la skill di test wasm"
        weight: 0.95
      - text: "avvia il loader wasm"
        weight: 0.95
      - text: "avvia il caricatore wasm"
        weight: 0.95
      - text: "avvia il modulo wasm"
        weight: 0.95
      - text: "avvia la skill di test wasm"
        weight: 0.95
      - text: "esegui il test del wasm"
        weight: 0.95
      - text: "prova il modulo wasm echo"
        weight: 0.95
      - text: "prova la skill wasm echo"
        weight: 0.95
      - text: "prova il caricatore wasm"
        weight: 0.95
      - text: "prova il modulo wasm"
        weight: 0.95
      - text: "prova l'echo wasm"
        weight: 0.95
      - text: "prova la skill di test wasm"
        weight: 0.95
      - text: "testa wasm echo"
        weight: 0.95
      - text: "testa il modulo wasm echo"
        weight: 0.95
      - text: "testa la skill wasm echo"
        weight: 0.95
      - text: "testa il loader wasm"
        weight: 0.95
      - text: "testa il caricatore wasm"
        weight: 0.95
      - text: "testa il modulo wasm"
        weight: 0.95
      - text: "testa l'echo wasm"
        weight: 0.95
      - text: "testa la skill di test wasm"
        weight: 0.95
      - text: "esegui wasm echo"
        weight: 0.95
      - text: "esegui il modulo wasm echo"
        weight: 0.95
      - text: "esegui l'echo wasm"
        weight: 0.95
      - text: "lancia wasm echo"
        weight: 0.95
      - text: "lancia il modulo wasm echo"
        weight: 0.95
      - text: "lancia la skill wasm echo"
        weight: 0.95
      - text: "lancia l'echo wasm"
        weight: 0.95
      - text: "avvia wasm echo"
        weight: 0.95
      - text: "avvia il modulo wasm echo"
        weight: 0.95
      - text: "avvia la skill wasm echo"
        weight: 0.95
      - text: "avvia l'echo wasm"
        weight: 0.95
      - text: "fai partire wasm echo"
        weight: 0.95
      - text: "fai partire il modulo wasm echo"
        weight: 0.95
      - text: "fai partire la skill wasm echo"
        weight: 0.95
      - text: "fai partire il loader wasm"
        weight: 0.95
      - text: "fai partire il caricatore wasm"
        weight: 0.95
      - text: "fai partire il modulo wasm"
        weight: 0.85
      - text: "fai partire l'echo wasm"
        weight: 0.95
      - text: "fai partire la skill di test wasm"
        weight: 0.95
      - text: "verifica wasm echo"
        weight: 0.95
      - text: "verifica il modulo wasm echo"
        weight: 0.95
      - text: "verifica la skill wasm echo"
        weight: 0.95
      - text: "verifica il loader wasm"
        weight: 0.95
      - text: "verifica il caricatore wasm"
        weight: 0.95
      - text: "verifica il modulo wasm"
        weight: 0.95
      - text: "verifica l'echo wasm"
        weight: 0.95
      - text: "verifica la skill di test wasm"
        weight: 0.95
      - text: "fai girare wasm echo"
        weight: 0.95
      - text: "fai girare il modulo wasm echo"
        weight: 0.95
      - text: "fai girare la skill wasm echo"
        weight: 0.95
      - text: "fai girare il loader wasm"
        weight: 0.95
      - text: "fai girare il caricatore wasm"
        weight: 0.95
      - text: "fai girare il modulo wasm"
        weight: 0.95
      - text: "fai girare l'echo wasm"
        weight: 0.95
      - text: "fai girare la skill di test wasm"
        weight: 0.95
      - text: "carica wasm echo"
        weight: 0.95
      - text: "carica il modulo wasm echo"
        weight: 0.95
      - text: "carica la skill wasm echo"
        weight: 0.95
      - text: "carica il loader wasm"
        weight: 0.95
      - text: "carica il caricatore wasm"
        weight: 0.95
      - text: "carica il modulo wasm"
        weight: 0.95
      - text: "carica l'echo wasm"
        weight: 0.95
      - text: "carica la skill di test wasm"
        weight: 0.95
      - text: "attiva wasm echo"
        weight: 0.95
      - text: "attiva il modulo wasm echo"
        weight: 0.95
      - text: "attiva la skill wasm echo"
        weight: 0.95
      - text: "attiva il loader wasm"
        weight: 0.95
      - text: "attiva il caricatore wasm"
        weight: 0.95
      - text: "attiva il modulo wasm"
        weight: 0.95
      - text: "attiva l'echo wasm"
        weight: 0.95
      - text: "attiva la skill di test wasm"
        weight: 0.95
      - text: "dammi il saluto del wasm"
        weight: 0.95
      - text: "fammi vedere che il wasm funziona"
        weight: 0.85
      - text: "controlla che il loader wasm risponda"
        weight: 0.95
      - text: "voglio testare il loader wasm"
        weight: 0.95
      - text: "modulo wasm di prova"
        weight: 0.95
      - text: "echo di prova wasm"
        weight: 0.95
      - text: "il wasm dice ciao"
        weight: 0.95
      - text: "fammi sentire il saluto wasm"
        weight: 0.95
      - text: "fai un test del modulo wasm"
        weight: 0.95
      - text: "provami il caricatore wasm"
        weight: 0.95
      - text: "dammi il saluto dal wasm echo"
        weight: 0.95
      - text: "controlla il modulo wasm echo"
        weight: 0.95
      - text: "wasm echo"
        weight: 0.95
      - text: "prova wasm echo"
        weight: 0.95
      - text: "prova il loader wasm"
        weight: 0.95
      - text: "esegui la skill wasm echo"
        weight: 0.95
      - text: "saluto dal modulo wasm"
        weight: 0.95
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# WASM Echo (Italiano)

Skill di riferimento che esiste unicamente per verificare che il loader WASM funzioni end-to-end. Un modulo Rust SDK minimale (`src/lib.rs`, compilato con `build.sh`) che esporta la superficie ABI v1 (`memory`, `ari_alloc`, `score`, `execute`). Restituisce la stringa `greeting` risolta per lingua da `strings/{locale}.json` tramite `ari::t()` — l'esempio canonico di localizzazione dell'output di una skill WASM ("wasm hello" in inglese, "ciao da wasm" in italiano).
