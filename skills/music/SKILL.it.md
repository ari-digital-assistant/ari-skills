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
    version: "0.1.1"
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
      - text: "metti su {artist}"
        weight: 0.6
        args:
          query: "{artist}"
      - text: "ascolta {artist}"
        weight: 0.95
        args:
          query: "{artist}"
      - text: "riproduci {artist}"
        weight: 0.95
        args:
          query: "{artist}"
      - text: "voglio ascoltare {artist}"
        weight: 0.95
        args:
          query: "{artist}"
      - text: "voglio sentire {artist}"
        weight: 0.75
        args:
          query: "{artist}"
      - text: "fammi sentire {artist}"
        weight: 0.75
        args:
          query: "{artist}"
      - text: "mettimi {artist}"
        weight: 0.95
        args:
          query: "{artist}"
      - text: "vorrei ascoltare {artist}"
        weight: 0.95
        args:
          query: "{artist}"
      - text: "puoi mettere {artist}"
        weight: 0.6
        args:
          query: "{artist}"
      - text: "metti {artist}"
        weight: 0.6
        args:
          query: "{artist}"
      - text: "metti su {artist} per favore"
        weight: 0.6
        args:
          query: "{artist}"
      - text: "metti su {artist} adesso"
        weight: 0.6
        args:
          query: "{artist}"
      - text: "fammi ascoltare {artist}"
        weight: 0.95
        args:
          query: "{artist}"
      - text: "senti {artist}"
        weight: 0.6
        args:
          query: "{artist}"
      - text: "metti su un po' di {genre}"
        weight: 0.6
        args:
          query: "{genre}"
      - text: "ascolta un po' di {genre}"
        weight: 0.95
        args:
          query: "{genre}"
      - text: "mettimi un po' di {genre}"
        weight: 0.75
        args:
          query: "{genre}"
      - text: "fammi sentire un po' di {genre}"
        weight: 0.75
        args:
          query: "{genre}"
      - text: "vorrei un po' di {genre}"
        weight: 0.6
        args:
          query: "{genre}"
      - text: "riproduci un po' di {genre}"
        weight: 0.95
        args:
          query: "{genre}"
      - text: "voglio sentire un po' di {genre}"
        weight: 0.75
        args:
          query: "{genre}"
      - text: "metti su {artist} su spotify"
        weight: 0.95
        args:
          query: "{artist}"
          service: "spotify"
      - text: "ascolta {artist} su spotify"
        weight: 0.95
        args:
          query: "{artist}"
          service: "spotify"
      - text: "riproduci {artist} su spotify"
        weight: 0.95
        args:
          query: "{artist}"
          service: "spotify"
      - text: "riproduci {artist} su apple music"
        weight: 0.95
        args:
          query: "{artist}"
          service: "apple_music"
      - text: "metti su {artist} su apple music"
        weight: 0.95
        args:
          query: "{artist}"
          service: "apple_music"
      - text: "ascolta {artist} su tidal"
        weight: 0.95
        args:
          query: "{artist}"
          service: "tidal"
      - text: "metti su {artist} su deezer"
        weight: 0.95
        args:
          query: "{artist}"
          service: "deezer"
      - text: "riproduci {artist} su amazon music"
        weight: 0.95
        args:
          query: "{artist}"
          service: "amazon_music"
      - text: "ascolta {artist} su amazon music"
        weight: 0.95
        args:
          query: "{artist}"
          service: "amazon_music"
      - text: "metti su {genre} su tidal"
        weight: 0.95
        args:
          query: "{genre}"
          service: "tidal"
      - text: "metti su qualcosa di rilassante"
        weight: 0.85
        args:
          query: "musica rilassante"
      - text: "metti su qualcosa di allegro"
        weight: 0.85
        args:
          query: "musica allegra"
      - text: "fammi sentire qualcosa di allegro"
        weight: 0.85
        args:
          query: "musica allegra"
      - text: "metti su qualcosa di tranquillo"
        weight: 0.85
        args:
          query: "musica rilassante"
      - text: "metti su qualcosa da ballare"
        weight: 0.85
        args:
          query: "musica dance"
      - text: "pausa"
        weight: 0.95
        args:
          action: "pause"
      - text: "metti in pausa"
        weight: 0.95
        args:
          action: "pause"
      - text: "ferma la musica"
        weight: 0.95
        args:
          action: "stop"
      - text: "basta musica"
        weight: 0.95
        args:
          action: "stop"
      - text: "riprendi"
        weight: 0.95
        args:
          action: "resume"
      - text: "riprendi la musica"
        weight: 0.95
        args:
          action: "resume"
      - text: "prossima"
        weight: 0.95
        args:
          action: "next"
      - text: "cambia canzone"
        weight: 0.95
        args:
          action: "next"
      - text: "torna indietro"
        weight: 0.95
        args:
          action: "previous"
      - text: "canzone precedente"
        weight: 0.95
        args:
          action: "previous"
      - text: "alza il volume"
        weight: 0.95
        args:
          action: "volume_up"
      - text: "più forte"
        weight: 0.75
        args:
          action: "volume_up"
      - text: "abbassa il volume"
        weight: 0.95
        args:
          action: "volume_down"
      - text: "muto"
        weight: 0.95
        args:
          action: "mute"
      - text: "imposta il volume al 40%"
        weight: 0.95
        args:
          action: "volume_set"
          level: "40"
      - text: "metti il volume al 70%"
        weight: 0.95
        args:
          action: "volume_set"
          level: "70"
      - text: "ascolta i {query}"
        weight: 0.95
        args:
          query: "{query}"
      - text: "metti su {query} su {service}"
        weight: 0.6
        args:
          query: "{query}"
          service: "{service}"
      - text: "ascolta {query} su {service}"
        weight: 0.95
        args:
          query: "{query}"
          service: "{service}"
      - text: "ascolta del {query}"
        weight: 0.95
        args:
          query: "{query}"
      - text: "riproduci {query} su {service}"
        weight: 0.95
        args:
          query: "{query}"
          service: "{service}"
      - text: "metti su qualcosa dei {query}"
        weight: 0.6
        args:
          query: "{query}"
      - text: "imposta il volume al {level}%"
        weight: 0.95
        args:
          action: "volume_set"
          level: "{level}"
      - text: "vorrei ascoltare i {query}"
        weight: 0.95
        args:
          query: "{query}"
      - text: "un po' di {query} per favore"
        weight: 0.6
        args:
          query: "{query}"
      - text: "torna alla canzone di prima"
        weight: 0.95
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
