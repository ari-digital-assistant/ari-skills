---
# `name` must match the directory (`weather/`) — it's the stable
# system identifier, not a display field. Per-locale display strings
# live in `description` (below) and the markdown body. Don't translate
# this.
name: weather
description: Meteo attuale, previsioni e condizioni come probabilità di pioggia, vento, umidità e UV — per la tua posizione attuale o per qualsiasi luogo tu indichi.
license: MIT
metadata:
  ari:
    id: dev.heyari.weather
    version: "0.2.1"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [http, location, storage_kv]
    languages: [en, it]
    specificity: high
    matching:
      # I pattern vengono confrontati con l'input POST-NORMALIZZATO: in minuscolo,
      # con le elisioni sostituite da uno spazio (`l'ora` → `l ora`) prima che
      # il motore applichi la regex. L'espansione delle contrazioni riguarda
      # solo l'inglese: per `it` la normalizzazione usa
      # `strip_italian_elisions`, non `expand_english_contractions`.
      # La normalizzazione NON rimuove gli accenti (`è`/`à` superano
      # il filtro `is_alphanumeric`, che riconosce i caratteri Unicode). Il motivo
      # di `piove(ra|rà)?` è un altro: gli utenti e i sistemi STT spesso omettono
      # l'accento durante la digitazione o la trascrizione, quindi il pattern copre
      # `piove`, `piovera` e `pioverà`.
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
        - regex: "\\bumidit(a|à)\\b"
          weight: 0.8
      custom_score: false
    # Gli esempi portano `args` che nominano i due slot: `location`
    # (stringa vuota = usa il GPS) e `when`. I valori di `when` restano
    # i token INGLESI (now | today | tomorrow | this week): è la skill
    # a mappare quei token.
    examples:
      - text: "Che tempo c'è fuori in questo momento?"
        weight: 0.85
        args:
          location: ""
          when: "now"
      - text: "Mi dici il meteo di oggi?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "Vorrei conoscere le condizioni meteo attuali."
        weight: 0.95
        args:
          location: ""
          when: "now"
      - text: "Dammi un aggiornamento sul tempo qui."
        weight: 0.75
        args:
          location: ""
          when: "now"
      - text: "Com'è il tempo dalle mie parti?"
        weight: 0.95
        args:
          location: ""
          when: "now"
      - text: "Controlla la temperatura esterna adesso."
        weight: 0.95
        args:
          location: ""
          when: "now"
      - text: "Che tempo c'è a {location}?"
        weight: 0.75
        args:
          location: "{location}"
          when: "now"
      - text: "Quali sono le condizioni meteo a {location}?"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "Com'è il cielo a {location} in questo momento?"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "Fa caldo o freddo a {location} adesso?"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "Qual è la temperatura esterna a {location}?"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "Sta piovendo a {location}?"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "C'è il sole a {location} in questo momento?"
        weight: 0.85
        args:
          location: "{location}"
          when: "now"
      - text: "Quanto vento c'è a {location} adesso?"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "Com'è l'umidità a {location}?"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "Qual è l'indice UV attuale a {location}?"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "Fammi vedere le previsioni meteo per oggi."
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "Pioverà nel corso della giornata?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "Qual è la probabilità di pioggia per oggi?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "È previsto vento oggi?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "Quanto sarà forte il vento oggi?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "Che livello di umidità è previsto oggi?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "Quanto sarà alto l'indice UV oggi?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "Quali temperature massime e minime avremo oggi?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "Farà caldo durante la giornata?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "Oggi farà abbastanza freddo da mettere il cappotto?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "Mi conviene prendere l'ombrello oggi?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "Serve una giacca pesante per il tempo di oggi?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "Posso stendere il bucato fuori oggi o rischia di piovere?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "È una giornata da spiaggia oggi?"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "Com'è il meteo a {location} oggi?"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "C'è rischio di pioggia a {location} oggi?"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "Che vento è previsto a {location} oggi?"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "Controlla l'indice UV a {location} per oggi."
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "Che temperature sono previste a {location} oggi?"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "Vorrei le previsioni meteo per domani."
        weight: 0.95
        args:
          location: ""
          when: "tomorrow"
      - text: "Come sarà il tempo a {location} domani?"
        weight: 0.85
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "Domani dobbiamo aspettarci pioggia?"
        weight: 0.95
        args:
          location: ""
          when: "tomorrow"
      - text: "A {location} pioverà domani?"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "Dovrò portarmi l'ombrello domani?"
        weight: 0.95
        args:
          location: ""
          when: "tomorrow"
      - text: "Domani servirà il cappotto per il freddo?"
        weight: 0.95
        args:
          location: ""
          when: "tomorrow"
      - text: "Ci sarà una giornata di sole domani?"
        weight: 0.95
        args:
          location: ""
          when: "tomorrow"
      - text: "Che vento farà domani?"
        weight: 0.95
        args:
          location: ""
          when: "tomorrow"
      - text: "Come sarà l'umidità domani?"
        weight: 0.95
        args:
          location: ""
          when: "tomorrow"
      - text: "Che indice UV è previsto per domani?"
        weight: 0.95
        args:
          location: ""
          when: "tomorrow"
      - text: "Quanti gradi sono previsti per domani?"
        weight: 0.95
        args:
          location: ""
          when: "tomorrow"
      - text: "Il tempo permetterà di pranzare fuori domani?"
        weight: 0.95
        args:
          location: ""
          when: "tomorrow"
      - text: "Domani sarà bel tempo per andare al mare?"
        weight: 0.95
        args:
          location: ""
          when: "tomorrow"
      - text: "Nevicherà a {location} domani?"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "Mostrami le previsioni per tutta la settimana."
        weight: 0.95
        args:
          location: ""
          when: "this week"
      - text: "Come si metterà il tempo nel corso di questa settimana?"
        weight: 0.95
        args:
          location: ""
          when: "this week"
      - text: "Ci saranno giornate di pioggia questa settimana?"
        weight: 0.95
        args:
          location: ""
          when: "this week"
      - text: "Come cambieranno le temperature a {location} questa settimana?"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "È previsto molto vento a {location} questa settimana?"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "C'è neve in arrivo a {location} questa settimana?"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "che tempo fa"
        weight: 0.6
        args:
          location: ""
          when: "now"
      - text: "che tempo fa a {location}"
        weight: 0.6
        args:
          location: "{location}"
          when: "now"
      - text: "meteo a {location} domani"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "previsioni per questa settimana"
        weight: 0.95
        args:
          location: ""
          when: "this week"
      - text: "pioverà oggi"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "c'è vento"
        weight: 0.95
        args:
          location: ""
          when: "now"
      - text: "qual è l'indice uv"
        weight: 0.95
        args:
          location: ""
          when: "now"
      - text: "c'è umidità oggi"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "mi serve il cappotto oggi"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "quanto fa caldo fuori"
        weight: 0.95
        args:
          location: ""
          when: "now"
      - text: "devo portare l'ombrello domani"
        weight: 0.85
        args:
          location: ""
          when: "tomorrow"
      - text: "farà freddo a {location} domani"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "serve la crema solare oggi"
        weight: 0.6
        args:
          location: ""
          when: "today"
      - text: "c'è il sole a {location}"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "nevicherà questa settimana"
        weight: 0.95
        args:
          location: ""
          when: "this week"
      - text: "quanti gradi faranno domani"
        weight: 0.85
        args:
          location: ""
          when: "tomorrow"
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
probabilità di pioggia, umidità e indice UV. Chiedi che tempo fa dove ti
trovi — la skill usa una posizione approssimativa del dispositivo —
oppure indica un luogo qualsiasi ("meteo a tokyo").

Le condizioni attuali vengono riassunte con una frase principale, seguita
dai dettagli che meritano di essere detti: la probabilità di pioggia per
le ore restanti della giornata (con l'ora di punta, quando la pioggia si
concentra invece di cadere tutto il giorno), direzione e velocità del
vento, e umidità. In una giornata calma e asciutta resta solo la frase
principale.

## Frasi supportate

- `che tempo fa` — condizioni attuali, posizione corrente
- `che tempo fa a tokyo` — condizioni attuali, luogo indicato
- `meteo a roma domani` — luogo indicato, giorno successivo
- `previsioni per questa settimana` — panoramica su più giorni
- `pioverà oggi` — domanda sulle precipitazioni
- `c'e vento` — domanda sul vento, riferita al giorno richiesto
- `qual e l'indice uv` — domanda sull'indice UV
- `c'e umidità oggi` — domanda sull'umidità

## Argomenti

La skill estrae due slot dalla frase:

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
