---
name: alarm
description: Imposta le sveglie del dispositivo passandole alla tua app Orologio. Capisce gli orari ("imposta una sveglia per le 7"), le etichette ("sveglia palestra alle 5 e mezza") e la ricorrenza ("svegliami alle 6:30 ogni giorno feriale"). Apre l'app Orologio per modificare o elencare le sveglie.
license: MIT
metadata:
  ari:
    id: dev.heyari.alarm
    version: "0.1.1"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [alarm]
    languages: [en, it]
    specificity: high
    matching:
      patterns:
        - regex: "\\b(imposta|crea|metti|aggiungi)\\b.*\\bsveglia\\b"
          weight: 0.95
        - regex: "\\bsvegliami\\b.*\\b(alle|alla|a)\\b"
          weight: 0.9
        - regex: "\\bsveglia\\b.*\\b(per|alle|alla)\\b"
          weight: 0.85
        - regex: "\\b(cancella|elimina|rimuovi|togli|disattiva|ferma)\\b.*\\bsveglia\\b"
          weight: 0.9
        - regex: "\\bquali sveglie\\b|\\belenca.*\\bsveglia|\\bche sveglie ho\\b"
          weight: 0.9
      custom_score: false
    examples:
      - text: "imposta una sveglia per le 7"
      - text: "imposta una sveglia per le 6 30 ogni giorno feriale"
      - text: "svegliami alle sei e mezza"
      - text: "sveglia palestra alle 5 45"
      - text: "imposta una sveglia per le 8 il sabato e la domenica"
      - text: "cancella la mia sveglia delle 7"
      - text: "che sveglie ho"
      - text: "disattiva la sveglia"
      # Frasi oblique che i pattern qui sopra non intercettano di proposito:
      # sono quelle che il router vede davvero in produzione.
      - text: "domani devo alzarmi alle sei"
      - text: "non farmi dormire oltre le otto"
      - text: "devo essere in piedi alle cinque e mezza"
      - text: "mi devo alzare prestissimo per il treno delle sei"
      - text: "tirami giù dal letto alle sette"
      - text: "voglio essere svegliato alle sei e un quarto"
      - text: "ho la riunione alle otto svegliami un'ora prima"
      - text: "fammi alzare alle cinque domani mattina"
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Sveglia

Imposta le sveglie del dispositivo passandole all'app Orologio della piattaforma.
