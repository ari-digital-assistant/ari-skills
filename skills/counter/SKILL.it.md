---
name: counter
description: Conta quante volte gli hai chiesto di contare, mantenendo il valore tra una chiamata e l'altra. Contatore ASCII a una cifra che torna da 9 a 1. Skill WASM di riferimento per gli import host storage_kv.
license: MIT
metadata:
  ari:
    id: dev.heyari.counter
    version: "0.1.1"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.1"
    capabilities: [storage_kv]
    languages: [en, it]
    specificity: high
    matching:
      # Ancorati, non una keyword `conta` nuda: come keyword rivendicava
      # qualsiasi frase contenente la parola — "conta 12 minuti per me" è
      # un timer, e questa skill rispondeva con una cifra.
      patterns:
        - regex: "^conta$"
          weight: 0.95
        - regex: "\\bcontatore\\b"
          weight: 0.95
    examples:
      - text: "conta"
      - text: "aggiungine uno"
      - text: "incrementa il contatore"
      - text: "aumenta il contatore"
      - text: "aggiungi uno al contatore"
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# Counter (Italiano)

Skill WASM di riferimento per gli import host `ari::storage_get` e `ari::storage_set`. Ogni chiamata incrementa una singola cifra ASCII memorizzata sotto la chiave `counter`. Persiste tra le invocazioni della CLI perché il file di storage risiede su disco.
