---
# `name` must match the directory (`weather/`) — it's the stable
# system identifier, not a display field. Per-locale display strings
# live in `description` (below) and the markdown body. Don't translate
# this.
name: weather
description: Meteo attuale, previsioni e condizioni come vento, pioggia e UV — per la tua posizione attuale o per qualsiasi luogo tu indichi.
license: MIT
metadata:
  ari:
    id: dev.heyari.weather
    version: "0.1.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [http, location, storage_kv]
    languages: [en, it]
    specificity: high
    matching:
      # Pattern confrontati con l'input POST-NORMALIZZATO: minuscolo, con
      # apostrofi/contrazioni espansi prima che l'engine esegua la regex.
      # In italiano la normalizzazione può rimuovere l'accento, quindi
      # `piove(ra|rà)?` copre `piove`, `piovera` e `pioverà`.
      patterns:
        - regex: "\\b(tempo|meteo)\\b"
          weight: 0.95
        - regex: "\\bprevisioni\\b"
          weight: 0.9
        - regex: "\\bpiove(ra|rà)?\\b"
          weight: 0.9
        - regex: "\\b(vento|ventoso)\\b"
          weight: 0.75
        - regex: "\\b(raggi )?uv\\b"
          weight: 0.8
      custom_score: false
    # Gli esempi portano `args` perché FunctionGemma impari a estrarre i
    # due slot: `location` (stringa vuota = usa il GPS) e `when`. I valori
    # di `when` restano i token INGLESI (now | today | tomorrow | this
    # week): il router della skill mappa quei token, e il modello impara
    # a emetterli dalle frasi italiane.
    examples:
      - text: "che tempo fa"
        args:
          location: ""
          when: "now"
      - text: "che tempo fa a tokyo"
        args:
          location: "tokyo"
          when: "now"
      - text: "meteo a roma domani"
        args:
          location: "roma"
          when: "tomorrow"
      - text: "previsioni per questa settimana"
        args:
          location: ""
          when: "this week"
      - text: "pioverà oggi"
        args:
          location: ""
          when: "today"
      - text: "c'e vento"
        args:
          location: ""
          when: "now"
      - text: "qual e l'indice uv"
        args:
          location: ""
          when: "now"
    settings:
      - key: units
        label: Unità
        type: select
        default: auto
        options:
          - value: auto
            label: Automatico
          - value: metric
            label: Metrico (°C, km/h)
          - value: imperial
            label: Imperiale (°F, mph)
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Meteo

Condizioni attuali e previsioni, oltre a domande puntuali su vento,
probabilità di pioggia e indice UV. Chiedi che tempo fa dove ti trovi —
la skill usa una posizione approssimativa del dispositivo — oppure indica
un luogo qualsiasi ("meteo a tokyo").

## Frasi supportate

- `che tempo fa` — condizioni attuali, posizione corrente
- `che tempo fa a tokyo` — condizioni attuali, luogo indicato
- `meteo a roma domani` — luogo indicato, giorno successivo
- `previsioni per questa settimana` — panoramica su più giorni
- `pioverà oggi` — domanda sulle precipitazioni
- `c'e vento` — domanda sul vento
- `qual e l'indice uv` — domanda sull'indice UV

## Argomenti estratti

Il router estrae due slot:

- `location` — il nome del luogo, oppure una stringa vuota per usare la
  posizione approssimativa del dispositivo tramite la capability
  `location`.
- `when` — uno tra `now`, `today`, `tomorrow` o `this week` (token
  inglesi). Predefinito a `now` quando la frase non contiene un orario.

## Impostazioni

- **Unità** — `Automatico` (segue la lingua del dispositivo), `Metrico`
  (°C, km/h), o `Imperiale` (°F, mph).

## Backend

Le previsioni provengono, tramite la capability `http`, da provider senza
chiave (Open-Meteo), con risultati memorizzati brevemente in cache tramite
`storage_kv` per evitare richieste ripetute all'API.
