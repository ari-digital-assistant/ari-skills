---
name: alarm
description: Imposta le sveglie del dispositivo passandole alla tua app Orologio. Capisce gli orari ("imposta una sveglia per le 7"), le etichette ("sveglia palestra alle 5 e mezza") e la ricorrenza ("svegliami alle 6:30 ogni giorno feriale"). Apre l'app Orologio per modificare o elencare le sveglie.
license: MIT
metadata:
  ari:
    id: dev.heyari.alarm
    version: "0.1.2"
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
      - text: "imposta una sveglia per le {time}"
        weight: 0.75
      - text: "imposta una sveglia per le {time} {time2}"
        weight: 0.75
      - text: "metti una sveglia alle {time}"
        weight: 0.75
      - text: "metti la sveglia alle {time}"
        weight: 0.75
      - text: "crea una sveglia per le {time} {time2}"
        weight: 0.95
      - text: "aggiungi una sveglia alle {time}"
        weight: 0.75
      - text: "svegliami alle {time}"
        weight: 0.95
      - text: "svegliami alle {time} {time2}"
        weight: 0.95
      - text: "svegliami alle {time} di mattina"
        weight: 0.95
      - text: "svegliami alle {time} domani"
        weight: 0.95
      - text: "voglio essere svegliato alle {time}"
        weight: 0.95
      - text: "voglio essere svegliata alle {time} {time2}"
        weight: 0.95
      - text: "puoi svegliarmi alle {time}"
        weight: 0.95
      - text: "mettimi la sveglia per le {time} {time2}"
        weight: 0.95
      - text: "imposta la sveglia della palestra alle {time}"
        weight: 0.95
      - text: "sveglia palestra alle {time} {time2}"
        weight: 0.95
      - text: "sveglia lavoro alle {time}"
        weight: 0.95
      - text: "imposta una sveglia per le {time} ogni giorno feriale"
        weight: 0.85
      - text: "imposta una sveglia alle {time} tutti i giorni"
        weight: 0.85
      - text: "sveglia alle {time} da lunedì a venerdì"
        weight: 0.95
      - text: "imposta una sveglia per le {time} il sabato"
        weight: 0.85
      - text: "metti una sveglia alle {time} {time2} ogni mattina"
        weight: 0.85
      - text: "domani devo alzarmi alle {time}"
        weight: 0.95
      - text: "devo alzarmi presto domani alle {time}"
        weight: 0.95
      - text: "devo essere in piedi alle {time} {time2}"
        weight: 0.95
      - text: "devo svegliarmi alle {time} per il treno"
        weight: 0.95
      - text: "non farmi dormire oltre le {time}"
        weight: 0.95
      - text: "assicurati che io sia sveglio alle {time}"
        weight: 0.95
      - text: "fammi alzare alle {time} domani mattina"
        weight: 0.95
      - text: "tirami giù dal letto alle {time}"
        weight: 0.95
      - text: "ho la riunione presto svegliami alle {time}"
        weight: 0.95
      - text: "voglio essere sveglio per le {time} {time2}"
        weight: 0.95
      - text: "mi serve una sveglia alle {time}"
        weight: 0.75
      - text: "puoi mettere la sveglia alle {time} {time2}"
        weight: 0.75
      - text: "programma la sveglia per le {time}"
        weight: 0.95
      - text: "imposta un allarme per le {time}"
        weight: 0.95
      - text: "metti un allarme alle {time} {time2}"
        weight: 0.95
      - text: "sveglia alle {time} in punto"
        weight: 0.95
      - text: "svegliami domani alle {time}"
        weight: 0.95
      - text: "svegliami presto alle {time}"
        weight: 0.95
      - text: "vorrei una sveglia per le {time} {time2}"
        weight: 0.75
      - text: "cancella la sveglia delle {time}"
        weight: 0.95
      - text: "elimina la mia sveglia delle {time}"
        weight: 0.95
      - text: "togli la sveglia delle {time} {time2}"
        weight: 0.95
      - text: "rimuovi la sveglia delle {time}"
        weight: 0.95
      - text: "disattiva la sveglia"
        weight: 0.95
      - text: "disattiva la sveglia delle {time}"
        weight: 0.95
      - text: "spegni la sveglia"
        weight: 0.95
      - text: "che sveglie ho"
        weight: 0.95
      - text: "quali sveglie ho impostato"
        weight: 0.95
      - text: "elenca le mie sveglie"
        weight: 0.95
      - text: "imposta una sveglia per le 7"
        weight: 0.75
      - text: "imposta una sveglia per le 6 30 ogni giorno feriale"
        weight: 0.75
      - text: "svegliami alle sei e mezza"
        weight: 0.95
      - text: "sveglia palestra alle 5 45"
        weight: 0.95
      - text: "imposta una sveglia per le 8 il sabato e la domenica"
        weight: 0.95
      - text: "cancella la mia sveglia delle 7"
        weight: 0.95
      - text: "domani devo alzarmi alle sei"
        weight: 0.95
      - text: "non farmi dormire oltre le otto"
        weight: 0.95
      - text: "devo essere in piedi alle cinque e mezza"
        weight: 0.75
      - text: "mi devo alzare prestissimo per il treno delle sei"
        weight: 0.85
      - text: "tirami giù dal letto alle sette"
        weight: 0.85
      - text: "voglio essere svegliato alle sei e un quarto"
        weight: 0.95
      - text: "ho la riunione alle otto svegliami un'ora prima"
        weight: 0.95
      - text: "fammi alzare alle cinque domani mattina"
        weight: 0.85
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Sveglia

Imposta le sveglie del dispositivo passandole all'app Orologio della piattaforma.
