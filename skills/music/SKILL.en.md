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
    languages: [en]
    specificity: medium
    matching:
      patterns:
        - regex: "\\b(play|put on|listen to)\\b"
          weight: 0.9
    examples:
      - text: "play hotel california"
      - text: "play some pink floyd"
      - text: "put on comfortably numb"
      - text: "listen to the beatles"
      - text: "play hotel california on spotify"
    wasm:
      module: skill.wasm
      memory_limit_mb: 2
---

# Music

Plays music by name in the user's chosen music service.
