---
name: navigation
description: Starts navigation to a place by handing off to your maps app. Understands destinations ("take me to McDonald's", "navigate to Asda", "how do I get to the station") and "take me home". A setting chooses between your default maps app and turn-by-turn navigation.
license: MIT
metadata:
  ari:
    id: dev.heyari.navigation
    version: "0.1.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [navigation]
    languages: [en, it]
    specificity: high
    matching:
      patterns:
        - regex: "\\b(navigate|directions|route) to\\b"
          weight: 0.95
        - regex: "\\b(take|bring|get|drive) me to\\b"
          weight: 0.9
        - regex: "\\bhow do i get to\\b"
          weight: 0.9
        - regex: "\\b(show me the way|the way) to\\b"
          weight: 0.85
        - regex: "\\b(take|bring|get|drive) me home\\b"
          weight: 0.9
      custom_score: false
    examples:
      - text: "take me to mcdonalds"
        args:
          destination: "mcdonalds"
      - text: "navigate to asda"
        args:
          destination: "asda"
      - text: "directions to the train station"
        args:
          destination: "train station"
      - text: "how do i get to the airport"
        args:
          destination: "airport"
      - text: "show me the way to the museum"
        args:
          destination: "museum"
      - text: "take me home"
        args:
          destination: "home"
      - text: "take me to work"
        args:
          destination: "work"
    settings:
      - key: navigation_mode
        label: Navigation style
        type: select
        default: default_app
        help_text: "Turn-by-turn uses Google Maps on Android; default maps opens the place in whatever maps app you've set."
        options:
          - value: default_app
            label: Open in my default maps app
          - value: turn_by_turn
            label: Start turn-by-turn navigation
    wasm:
      module: skill.wasm
      memory_limit_mb: 2
---

# Navigation

Starts navigation to a destination by handing off to the platform maps app.
The maps app owns routing, the map, and live traffic.

## Supported utterances

- `take me to McDonald's` / `navigate to Asda` — navigate to a place
- `how do I get to the station` / `show me the way to the museum`
- `take me home` — navigate to home (resolved by your maps app's saved places)

## Notes

The `navigate` action is generic: the skill emits *what* to do; each frontend
decides *how*. On Android it maps to an `ACTION_VIEW` `geo:` intent (default
maps app) or `google.navigation:` (turn-by-turn), chosen by the
**Navigation style** setting.
