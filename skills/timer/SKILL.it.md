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
      - text: "imposta un timer per 10 minuti"
      - text: "imposta un timer per la pasta di 8 minuti"
      - text: "avvia un timer di 4 minuti per la pasta"
      - text: "quanto manca al mio timer della pasta"
      - text: "annulla il mio timer della pasta"
      - text: "quali timer ho"
      - text: "imposta un timer per 5 minuti e un altro per 15 minuti"
      # Frasi oblique che i pattern qui sopra non intercettano di proposito:
      # sono quelle che il router vede davvero in produzione.
      - text: "avvisami tra dieci minuti"
      - text: "mettimi otto minuti per la pasta"
      - text: "fammi sapere quando sono passati venti minuti"
      - text: "suona tra un quarto d'ora"
      - text: "quanto resta alla pasta"
      - text: "ancora quanto per le uova"
      - text: "non mi serve più il timer della pasta"
      - text: "che timer ho attivi"
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
