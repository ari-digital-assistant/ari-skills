---
name: home-assistant
description: Controls your Home Assistant smart home — turn devices on and off, set brightness or temperature, run scenes, check status, and ask where people are. Use for any smart-home or home-automation request.
license: MIT
metadata:
  ari:
    id: dev.heyari.homeassistant
    version: "0.1.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [http, authorize, storage_kv]
    languages: [en]
    specificity: medium
    matching:
      patterns:
        - regex: "\\b(turn|switch) (on|off)\\b"
          weight: 0.9
        - regex: "\\bturn (the |my )?.+ (on|off)\\b"
          weight: 0.9
        - regex: "\\b(dim|brighten)\\b"
          weight: 0.85
        - regex: "\\bset (the |my )?.+ (to|brightness|temperature)\\b"
          weight: 0.85
        - regex: "\\b(open|close|lock|unlock)\\b"
          weight: 0.8
        - regex: "\\b(activate|run) (the )?scene\\b"
          weight: 0.9
        - regex: "\\bwhere (is|are)\\b"
          weight: 0.75
        - keywords: [thermostat, lights]
          weight: 0.7
    examples:
      - text: "turn on the kitchen lights"
      - text: "turn off the bedroom lamp"
      - text: "set the living room to 21 degrees"
      - text: "dim the hallway lights to 30 percent"
      - text: "lock the front door"
      - text: "activate movie night scene"
      - text: "is the garage door open"
      - text: "where is keith"
    settings:
      - key: base_url
        label: "Home Assistant URL"
        type: text
        required: true
      - key: token
        label: "Long-lived access token"
        type: secret
        required: true
        validate: true
        depends_on: [base_url, token]
      - key: language
        label: "Voice command language (blank = app language)"
        type: text
        required: false
      - key: agent_id
        label: "Conversation agent entity (blank = HA default/local)"
        type: dynamic_select
        required: false
        depends_on: [base_url, token]
    fallback:
      requires_setting: base_url
    wasm:
      module: skill.wasm
      memory_limit_mb: 2
---

# Home Assistant

Bridges Ari to your Home Assistant server. Control utterances ("turn on the
kitchen lights", "set the bedroom to 21", "activate movie night") are forwarded
to HA's `conversation/process` API, which resolves the entities/areas and
replies in your language. "Where is <person>?" is answered by reading the
matching `person.*` entity's state. Person location is read directly, so it
works regardless of which entities are exposed to voice assistants.

**Setup:** enter your server URL (e.g. `http://homeassistant.local:8123` or your
Nabu Casa URL) and a long-lived access token from your HA profile page.
A `http://`/`.local`/LAN-IP URL only works when your device is on the home
network; use a Nabu Casa or external HTTPS URL for control while away.
