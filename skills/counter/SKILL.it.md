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
        weight: 0.95
      - text: "contami"
        weight: 0.95
      - text: "aggiungine uno al conteggio"
        weight: 0.95
      - text: "aggiungi uno al contatore"
        weight: 0.95
      - text: "incrementa il contatore"
        weight: 0.95
      - text: "aumenta il contatore"
        weight: 0.95
      - text: "alza il contatore"
        weight: 0.95
      - text: "fai salire il contatore"
        weight: 0.95
      - text: "segnane uno sul conteggio"
        weight: 0.95
      - text: "conta ancora"
        weight: 0.75
      - text: "conta di nuovo"
        weight: 0.75
      - text: "aggiorna il contatore"
        weight: 0.95
      - text: "spingi avanti il contatore"
        weight: 0.95
      - text: "fammi contare"
        weight: 0.95
      - text: "aggiungi al contatore"
        weight: 0.95
      - text: "incrementa"
        weight: 0.95
      - text: "conta uno in più"
        weight: 0.75
      - text: "segna un altro"
        weight: 0.95
      - text: "aumenta il conteggio di uno"
        weight: 0.95
      - text: "aggiungine un altro"
        weight: 0.95
      - text: "fai un altro conteggio"
        weight: 0.95
      - text: "conta su"
        weight: 0.95
      - text: "conteggia"
        weight: 0.95
      - text: "metti un altro sul contatore"
        weight: 0.95
      - text: "aggiungi {count} al contatore"
        weight: 0.95
      - text: "aumenta il contatore di {count}"
        weight: 0.95
      - text: "incrementa il contatore di {count}"
        weight: 0.95
      - text: "segna {count} sul contatore"
        weight: 0.95
      - text: "alza il contatore di {count}"
        weight: 0.95
      - text: "fai salire il contatore di {count}"
        weight: 0.95
      - text: "metti {count} sul contatore"
        weight: 0.95
      - text: "aggiungi {count} al conteggio"
        weight: 0.95
      - text: "più {count} sul contatore"
        weight: 0.95
      - text: "conta fino a {count}"
        weight: 0.95
      - text: "aggiorna il contatore di {count}"
        weight: 0.95
      - text: "spingi avanti il contatore di {count}"
        weight: 0.95
      - text: "aumenta di {count} il contatore"
        weight: 0.95
      - text: "incrementa di {count}"
        weight: 0.95
      - text: "fai salire il conteggio di {count}"
        weight: 0.95
      - text: "registra {count} in più sul conteggio"
        weight: 0.95
      - text: "aumenta il conteggio di {count}"
        weight: 0.95
      - text: "porta avanti il contatore di {count}"
        weight: 0.95
      - text: "aggiungine uno"
        weight: 0.95
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# Counter (Italiano)

Skill WASM di riferimento per gli import host `ari::storage_get` e `ari::storage_set`. Ogni chiamata incrementa una singola cifra ASCII memorizzata sotto la chiave `counter`. Persiste tra le invocazioni della CLI perché il file di storage risiede su disco.
