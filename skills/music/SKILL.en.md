---
name: music
description: Plays music by name in a music app, optionally on a named service.
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
      patterns:
        - regex: "\\b(play|put on|listen to)\\b"
          weight: 0.9
        - regex: "\\b(pause|resume|skip|next|previous|stop|mute|unmute|louder|quieter|volume)\\b"
          weight: 0.9
    examples:
      - text: "play hotel california"
        args:
          query: "hotel california"
      - text: "play some pink floyd"
        args:
          query: "pink floyd"
      - text: "put on comfortably numb"
        args:
          query: "comfortably numb"
      - text: "listen to the beatles"
        args:
          query: "the beatles"
      - text: "play hotel california on spotify"
        args:
          query: "hotel california"
          service: "spotify"
      - text: "put on some jazz"
        args:
          query: "jazz"
      - text: "play abbey road on apple music"
        args:
          query: "abbey road"
          service: "apple_music"
      - text: "listen to radiohead on tidal"
        args:
          query: "radiohead"
          service: "tidal"
      - text: "play something relaxing"
        args:
          query: "relaxing music"
      - text: "put on led zeppelin"
        args:
          query: "led zeppelin"
      - text: "play thriller on spotify"
        args:
          query: "thriller"
          service: "spotify"
      # Italian examples — routed here because play/put on/listen to
      # are English triggers; Italian routing is handled by SKILL.it.md.
      # These bilingual entries teach FunctionGemma the arg shape.
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
      - text: "pause"
        args:
          action: "pause"
      - text: "resume"
        args:
          action: "resume"
      - text: "next"
        args:
          action: "next"
      - text: "skip this song"
        args:
          action: "next"
      - text: "previous"
        args:
          action: "previous"
      - text: "stop the music"
        args:
          action: "stop"
      - text: "volume up"
        args:
          action: "volume_up"
      - text: "turn it down"
        args:
          action: "volume_down"
      - text: "set volume to 50%"
        args:
          action: "volume_set"
          level: 50
      - text: "mute"
        args:
          action: "mute"
    settings:
      - key: default_service
        label: Default music service
        type: select
        default: last_used
        help_text: "Which service to use when you don't say one. 'Last used' remembers your previous choice."
        options:
          - value: last_used
            label: Last used
          - value: ask
            label: Ask each time
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

# Music

Plays music by name in the user's chosen music service.
