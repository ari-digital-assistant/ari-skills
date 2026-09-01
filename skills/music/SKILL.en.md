---
name: music
description: Plays music by name in a music app, optionally on a named service.
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
      patterns:
        - regex: "\\b(play|put on|listen to)\\b"
          weight: 0.9
        - regex: "\\b(pause|resume|skip|next|previous|stop|mute|unmute|louder|quieter|volume)\\b"
          weight: 0.9
    examples:
      - text: "play {song}"
        weight: 0.95
        args:
          query: "{song}"
      - text: "put on the track {song}"
        weight: 0.75
        args:
          query: "{song}"
      - text: "listen to {song}"
        weight: 0.95
        args:
          query: "{song}"
      - text: "i want to hear {song}"
        weight: 0.75
        args:
          query: "{song}"
      - text: "can you play {song}"
        weight: 0.95
        args:
          query: "{song}"
      - text: "play {song} on {service}"
        weight: 0.95
        args:
          query: "{song}"
          service: "{service}"
      - text: "put on {song} on {service}"
        weight: 0.6
        args:
          query: "{song}"
          service: "{service}"
      - text: "stick the song {song} on"
        weight: 0.95
        args:
          query: "{song}"
      - text: "play me {song}"
        weight: 0.95
        args:
          query: "{song}"
      - text: "put the song {song} on"
        weight: 0.95
        args:
          query: "{song}"
      - text: "play some {artist}"
        weight: 0.95
        args:
          query: "{artist}"
      - text: "put on some {artist}"
        weight: 0.6
        args:
          query: "{artist}"
      - text: "i fancy some {artist}"
        weight: 0.95
        args:
          query: "{artist}"
      - text: "play a bit of {artist}"
        weight: 0.95
        args:
          query: "{artist}"
      - text: "play some {artist} on {service}"
        weight: 0.95
        args:
          query: "{artist}"
          service: "{service}"
      - text: "can we have some {artist}"
        weight: 0.55
        args:
          query: "{artist}"
      - text: "shuffle some {artist}"
        weight: 0.95
        args:
          query: "{artist}"
      - text: "i want to listen to {genre}"
        weight: 0.95
        args:
          query: "{genre}"
      - text: "put on a bit of {genre}"
        weight: 0.6
        args:
          query: "{genre}"
      - text: "some {genre} please"
        weight: 0.55
        args:
          query: "{genre}"
      - text: "throw on some {genre}"
        weight: 0.95
        args:
          query: "{genre}"
      - text: "play something {mood}"
        weight: 0.95
        args:
          query: "{mood}"
      - text: "put on something {mood}"
        weight: 0.75
        args:
          query: "{mood}"
      - text: "i want something {mood}"
        weight: 0.75
        args:
          query: "{mood}"
      - text: "pause"
        weight: 0.95
        args:
          action: "pause"
      - text: "pause the music"
        weight: 0.95
        args:
          action: "pause"
      - text: "resume"
        weight: 0.95
        args:
          action: "resume"
      - text: "resume the music"
        weight: 0.95
        args:
          action: "resume"
      - text: "keep playing"
        weight: 0.95
        args:
          action: "resume"
      - text: "next"
        weight: 0.95
        args:
          action: "next"
      - text: "skip this track"
        weight: 0.95
        args:
          action: "next"
      - text: "next song"
        weight: 0.95
        args:
          action: "next"
      - text: "go back a track"
        weight: 0.75
        args:
          action: "previous"
      - text: "previous"
        weight: 0.95
        args:
          action: "previous"
      - text: "play the last song again"
        weight: 0.95
        args:
          action: "previous"
      - text: "stop playing"
        weight: 0.95
        args:
          action: "stop"
      - text: "turn the music off"
        weight: 0.95
        args:
          action: "stop"
      - text: "turn the music up"
        weight: 0.95
        args:
          action: "volume_up"
      - text: "louder"
        weight: 0.95
        args:
          action: "volume_up"
      - text: "crank it up"
        weight: 0.95
        args:
          action: "volume_up"
      - text: "turn the music down"
        weight: 0.95
        args:
          action: "volume_down"
      - text: "quieter"
        weight: 0.95
        args:
          action: "volume_down"
      - text: "bit quieter please"
        weight: 0.95
        args:
          action: "volume_down"
      - text: "mute the music"
        weight: 0.95
        args:
          action: "mute"
      - text: "mute it"
        weight: 0.95
        args:
          action: "mute"
      - text: "set the volume to 50"
        weight: 0.95
        args:
          action: "volume_set"
          level: "50"
      - text: "set the volume to 30"
        weight: 0.95
        args:
          action: "volume_set"
          level: "30"
      - text: "set the volume to 70"
        weight: 0.95
        args:
          action: "volume_set"
          level: "70"
      - text: "put the volume at 20"
        weight: 0.95
        args:
          action: "volume_set"
          level: "20"
      - text: "turn the volume up to 80"
        weight: 0.95
        args:
          action: "volume_set"
          level: "80"
      - text: "put on {query}"
        weight: 0.6
        args:
          query: "{query}"
      - text: "play {query} on apple music"
        weight: 0.95
        args:
          query: "{query}"
          service: "apple_music"
      - text: "listen to {query} on {service}"
        weight: 0.95
        args:
          query: "{query}"
          service: "{service}"
      - text: "play something relaxing"
        weight: 0.95
        args:
          query: "relaxing music"
      - text: "{action}"
        weight: 0.55
        args:
          action: "{action}"
      - text: "skip this song"
        weight: 0.95
        args:
          action: "next"
      - text: "{action} the music"
        weight: 0.95
        args:
          action: "{action}"
      - text: "volume up"
        weight: 0.95
        args:
          action: "volume_up"
      - text: "turn it down"
        weight: 0.6
        args:
          action: "volume_down"
      - text: "set volume to {level}%"
        weight: 0.95
        args:
          action: "volume_set"
          level: "{level}"
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
