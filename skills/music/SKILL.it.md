---
# `name` must match the directory (`music/`) — it's the stable
# system identifier, not a display field. Per-locale display strings
# live in `description` (below) and the markdown body. Don't translate
# this.
name: music
description: Riproduce musica per nome in un'app musicale, facoltativamente su un servizio specificato.
license: MIT
metadata:
  ari:
    id: dev.heyari.music
    version: "0.1.0"
    author: Ari core team
    engine: ">=0.3"
    capabilities: [media_control, media_services, storage_kv]
    languages: [en, it]
    specificity: medium
    matching:
      # Pattern confrontati con l'input POST-NORMALIZZATO: minuscolo,
      # apostrofi/contrazioni rimossi prima che l'engine esegua la regex.
      patterns:
        - regex: "\\b(metti su|riproduci|ascolta)\\b"
          weight: 0.9
        - regex: "\\b(pausa|riprendi|prossima|successiva|avanti|salta|precedente|ferma|muto|silenzia|volume)\\b"
          weight: 0.9
    examples:
      - text: "metti su bohemian rhapsody"
        args:
          query: "bohemian rhapsody"
      - text: "ascolta i pink floyd"
        args:
          query: "pink floyd"
      - text: "metti su qualcosa di rilassante"
        args:
          query: "musica rilassante"
      - text: "metti su hotel california su spotify"
        args:
          query: "hotel california"
          service: "spotify"
      - text: "ascolta radiohead su tidal"
        args:
          query: "radiohead"
          service: "tidal"
      - text: "metti su abbey road su apple music"
        args:
          query: "abbey road"
          service: "apple_music"
      - text: "ascolta del jazz"
        args:
          query: "jazz"
      - text: "metti su led zeppelin"
        args:
          query: "led zeppelin"
      - text: "riproduci thriller su spotify"
        args:
          query: "thriller"
          service: "spotify"
      - text: "metti su qualcosa dei beatles"
        args:
          query: "beatles"
      - text: "riproduci comfortably numb"
        args:
          query: "comfortably numb"
      - text: "pausa"
        args:
          action: "pause"
      - text: "metti in pausa"
        args:
          action: "pause"
      - text: "riprendi"
        args:
          action: "resume"
      - text: "prossima"
        args:
          action: "next"
      - text: "torna indietro"
        args:
          action: "previous"
      - text: "ferma la musica"
        args:
          action: "stop"
      - text: "alza il volume"
        args:
          action: "volume_up"
      - text: "abbassa il volume"
        args:
          action: "volume_down"
      - text: "imposta il volume al 40%"
        args:
          action: "volume_set"
          level: 40
      - text: "muto"
        args:
          action: "mute"
      # Frasi oblique che i pattern qui sopra non intercettano di proposito:
      # sono quelle che il router vede davvero in produzione.
      - text: "vorrei ascoltare i queen"
        args:
          query: "queen"
      - text: "voglio sentire vasco rossi"
        args:
          query: "vasco rossi"
      - text: "fammi sentire qualcosa di allegro"
        args:
          query: "musica allegra"
      - text: "un po' di musica classica per favore"
        args:
          query: "musica classica"
      - text: "basta musica"
        args:
          action: "stop"
      - text: "cambia canzone"
        args:
          action: "next"
      - text: "più forte"
        args:
          action: "volume_up"
      - text: "torna alla canzone di prima"
        args:
          action: "previous"
    settings:
      - key: default_service
        label: Servizio musicale predefinito
        type: select
        default: last_used
        help_text: "Quale servizio usare quando non ne specifichi uno. 'Ultimo usato' ricorda la tua scelta precedente."
        options:
          - value: last_used
            label: Ultimo usato
          - value: ask
            label: Chiedi ogni volta
          - value: spotify
            label: Spotify
          - value: apple_music
            label: Apple Music
          - value: tidal
            label: Tidal
          - value: deezer
            label: Deezer
          - value: amazon_music
            label: Amazon Music
    wasm:
      module: skill.wasm
      memory_limit_mb: 2
---

# Musica

Riproduce musica per nome nel servizio musicale scelto dall'utente.
