---
# `name` must match the directory (`timer/`) — it's the stable system
# identifier, not a display field. Per-locale display strings live in
# `description` (below) and the markdown body. Don't translate this.
name: timer
description: Imposta, interroga e annulla timer con nome. Supporta frasi naturali come "imposta un timer per la pasta di 8 minuti". Gestisce più timer contemporaneamente.
license: MIT
metadata:
  ari:
    id: dev.heyari.timer
    version: "0.2.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [storage_kv, critical_alert]
    languages: [en, it]
    specificity: high
    matching:
      patterns:
        # Italian "imposta/avvia/metti/crea un timer". Patterns match
        # post-normalize_input text — lowercased, apostrophes stripped.
        - regex: "\\b(imposta|avvia|metti|crea) (un )?timer\\b"
          weight: 0.95
        - regex: "\\btimer .* (di|per) \\d+ (second|minut|or)"
          weight: 0.9
        - regex: "\\bquanto (tempo )?manca\\b"
          weight: 0.85
        - regex: "\\b(annulla|cancella) .*timer\\b"
          weight: 0.9
        - regex: "\\b(elenca|mostra) .*timer\\b"
          weight: 0.85
        - regex: "\\bquali .*timer\\b"
          weight: 0.85
      custom_score: false
    examples:
      - text: "imposta un timer di {minutes} minuti"
        weight: 0.95
      - text: "imposta un timer per {minutes} minuti"
        weight: 0.95
      - text: "avvia un timer di {minutes} minuti"
        weight: 0.95
      - text: "metti un timer di {minutes} minuti"
        weight: 0.95
      - text: "crea un timer da {minutes} minuti"
        weight: 0.95
      - text: "fai partire un timer di {minutes} minuti"
        weight: 0.95
      - text: "imposta un timer per {name} di {minutes} minuti"
        weight: 0.95
      - text: "avvia un timer di {minutes} minuti per {name}"
        weight: 0.95
      - text: "metti un timer di {minutes} minuti per {name}"
        weight: 0.95
      - text: "imposta il timer per {name}"
        weight: 0.95
      - text: "un timer da {minutes} minuti per {name}"
        weight: 0.95
      - text: "imposta un timer di {seconds} secondi"
        weight: 0.95
      - text: "avvia un timer di {seconds} secondi"
        weight: 0.95
      - text: "metti {minutes} minuti"
        weight: 0.95
      - text: "mettimi {minutes} minuti per {name}"
        weight: 0.95
      - text: "dammi {minutes} minuti per {name}"
        weight: 0.95
      - text: "avvisami tra {minutes} minuti"
        weight: 0.95
      - text: "avvisami tra {minutes} minuti per {name}"
        weight: 0.95
      - text: "fammi sapere quando sono passati {minutes} minuti"
        weight: 0.85
      - text: "conto alla rovescia di {minutes} minuti"
        weight: 0.95
      - text: "conto alla rovescia di {minutes} minuti per {name}"
        weight: 0.95
      - text: "fai un conto alla rovescia di {minutes} minuti"
        weight: 0.95
      - text: "suona tra {minutes} minuti"
        weight: 0.95
      - text: "parti con un timer di {minutes} minuti"
        weight: 0.95
      - text: "fai partire {minutes} minuti per {name}"
        weight: 0.85
      - text: "imposta un timer di {minutes} minuti e un altro di {minutes2} minuti"
        weight: 0.95
      - text: "metti un timer di {minutes} minuti e uno di {minutes2} minuti"
        weight: 0.95
      - text: "quanto manca al timer per {name}"
        weight: 0.95
      - text: "quanto resta al timer per {name}"
        weight: 0.95
      - text: "quanto manca per {name}"
        weight: 0.75
      - text: "ancora quanto per {name}"
        weight: 0.6
      - text: "quanto manca"
        weight: 0.75
      - text: "quanti minuti mancano al timer per {name}"
        weight: 0.95
      - text: "annulla il timer per {name}"
        weight: 0.95
      - text: "cancella il timer per {name}"
        weight: 0.95
      - text: "elimina il timer per {name}"
        weight: 0.95
      - text: "non mi serve più il timer per {name}"
        weight: 0.85
      - text: "togli il timer per {name}"
        weight: 0.95
      - text: "annulla il timer di {minutes} minuti"
        weight: 0.95
      - text: "cancella tutti i timer"
        weight: 0.95
      - text: "quali timer ho"
        weight: 0.95
      - text: "quali timer ho attivi"
        weight: 0.95
      - text: "che timer ho attivi"
        weight: 0.95
      - text: "elenca i miei timer"
        weight: 0.95
      - text: "mostrami i timer attivi"
        weight: 0.95
      - text: "quanti timer ho attivi"
        weight: 0.95
      - text: "imposta un timer per {name} da {minutes} minuti"
        weight: 0.95
      - text: "fai un timer di {minutes} minuti per {name}"
        weight: 0.95
      - text: "fai partire il timer per {name}"
        weight: 0.85
      - text: "ancora molto per {name}"
        weight: 0.6
      - text: "imposta un timer per 10 minuti"
        weight: 0.95
      - text: "imposta un timer per la pasta di 8 minuti"
        weight: 0.95
      - text: "avvia un timer di 4 minuti per la pasta"
        weight: 0.95
      - text: "quanto manca al mio timer della pasta"
        weight: 0.85
      - text: "annulla il mio timer della pasta"
        weight: 0.95
      - text: "imposta un timer per 5 minuti e un altro per 15 minuti"
        weight: 0.95
      - text: "avvisami tra dieci minuti"
        weight: 0.95
      - text: "mettimi otto minuti per la pasta"
        weight: 0.95
      - text: "fammi sapere quando sono passati venti minuti"
        weight: 0.85
      - text: "suona tra un quarto d'ora"
        weight: 0.95
      - text: "quanto resta alla pasta"
        weight: 0.75
      - text: "ancora quanto per le uova"
        weight: 0.6
      - text: "non mi serve più il timer della pasta"
        weight: 0.75
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Timer

Imposta e gestisce timer con nome.

## Frasi supportate

- `imposta un timer per 10 minuti` — timer anonimo
- `imposta un timer per la pasta di 8 minuti` — timer con nome
- `avvia un timer di 4 minuti per la pasta` — timer con nome (forma aggettivale)
- `imposta un timer per 5 minuti e un altro per 15 minuti` — creazione multipla
- `quanto manca al mio timer della pasta` — interrogazione
- `annulla il mio timer della pasta` / `cancella il mio timer della pasta` — annullamento
- `quali timer ho` / `elenca i miei timer` — elenco

## Note

Lo stato dei timer è persistito nel `storage_kv` di questa skill. I timer scaduti
vengono eliminati a ogni invocazione, quindi le voci orfane di un'app chiusa in
background si riparano da sole.
