---
name: home-assistant
description: Controls your Home Assistant smart home — turn devices on and off, set brightness or temperature, run scenes, check status, and ask where people are. Use for any smart-home or home-automation request.
license: MIT
metadata:
  ari:
    id: dev.heyari.homeassistant
    version: "0.3.1"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.4"
    capabilities: [http, authorize, storage_kv]
    languages: [en, it]
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
      - text: "open the {room} blinds"
        weight: 0.95
      - text: "run the good morning scene"
        weight: 0.85
      - text: "start the bedtime scene"
        weight: 0.95
      - text: "open the garage door"
        weight: 0.95
      - text: "turn on {entity}"
        weight: 0.6
      - text: "turn off {entity}"
        weight: 0.6
      - text: "switch on {entity}"
        weight: 0.95
      - text: "switch off {entity}"
        weight: 0.95
      - text: "turn on the {room} lights"
        weight: 0.95
      - text: "turn off the {room} lights"
        weight: 0.95
      - text: "dim the {room} lights"
        weight: 0.95
      - text: "dim the {room} lights to {percent} percent"
        weight: 0.95
      - text: "brighten the {room} lights"
        weight: 0.95
      - text: "set the {room} lights to {percent} percent"
        weight: 0.95
      - text: "set the {room} to {temperature} degrees"
        weight: 0.95
      - text: "set the thermostat to {temperature} degrees"
        weight: 0.95
      - text: "turn the heating up to {temperature}"
        weight: 0.95
      - text: "make it {temperature} degrees in the {room}"
        weight: 0.95
      - text: "put the {room} lights on"
        weight: 0.95
      - text: "can you switch off {entity}"
        weight: 0.95
      - text: "close the {room} blinds"
        weight: 0.95
      - text: "lock the front door"
        weight: 0.95
      - text: "unlock the front door"
        weight: 0.95
      - text: "is {entity} on"
        weight: 0.55
      - text: "is {entity} still on"
        weight: 0.95
      - text: "did i leave {entity} on"
        weight: 0.6
      - text: "turn {entity} back on"
        weight: 0.6
      - text: "shut off {entity}"
        weight: 0.95
      - text: "kill {entity}"
        weight: 0.95
      - text: "lights on in the {room}"
        weight: 0.95
      - text: "lights off in the {room}"
        weight: 0.95
      - text: "turn the {room} lights down a bit"
        weight: 0.85
      - text: "turn the {room} lights right down"
        weight: 0.95
      - text: "set the brightness of the {room} lights to {percent} percent"
        weight: 0.95
      - text: "drop the temperature to {temperature}"
        weight: 0.95
      - text: "bump the thermostat up to {temperature}"
        weight: 0.95
      - text: "cool the {room} down to {temperature}"
        weight: 0.95
      - text: "warm the {room} up to {temperature}"
        weight: 0.95
      - text: "activate movie night scene"
        weight: 0.95
      - text: "set the scene for movie night"
        weight: 0.95
      - text: "is the garage door open"
        weight: 0.95
      - text: "close the garage door"
        weight: 0.95
      - text: "where is dad"
        weight: 0.95
      - text: "where is mum"
        weight: 0.95
      - text: "where is everyone"
        weight: 0.95
      - text: "is anyone home"
        weight: 0.95
      - text: "turn everything off"
        weight: 0.75
      - text: "turn all the lights off"
        weight: 0.85
      - text: "turn off all the lights downstairs"
        weight: 0.95
      - text: "switch the {room} lamp on"
        weight: 0.95
      - text: "can you dim {entity}"
        weight: 0.95
      - text: "put {entity} on"
        weight: 0.6
      - text: "set {entity} to {percent} percent"
        weight: 0.75
      - text: "turn the fan on in the {room}"
        weight: 0.95
      - text: "is the {room} light on"
        weight: 0.95
      - text: "turn on the kitchen lights"
        weight: 0.95
      - text: "turn off the bedroom lamp"
        weight: 0.85
      - text: "set the living room to 21 degrees"
        weight: 0.75
      - text: "dim the hallway lights to 30 percent"
        weight: 0.95
      - text: "where is keith"
        weight: 0.6
    settings:
      - key: base_url
        label: "Home Assistant URL"
        type: text
        required: true
        keyboard: url
      - key: sign_in
        label: "Sign in with Home Assistant"
        type: action
        depends_on: [base_url]
      - key: agent_id
        label: "Conversation agent entity (blank = HA default/local)"
        type: dynamic_select
        required: false
        depends_on: [base_url]
      - key: token
        label: "Long-lived access token"
        type: secret
        required: false
        validate: true
        depends_on: [base_url, token]
        collapsed_group: "Use token authentication instead"
        help_text: "Create a long-lived access token in your Home Assistant profile (bottom of the page) and paste it here."
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
