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
      - text: "take me to {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "navigate to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "directions to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "how do i get to {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "show me the way to {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "get me to {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "drive me to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "bring me to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "route to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "i need directions to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "guide me to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "can you take me to {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "set a route to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "plot a course to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "put {destination} in the sat nav"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "fire up the sat nav for {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "i want to go to {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "lets head to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "whats the best way to {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "whats the quickest route to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "find me a route to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "i need to get to {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "help me get to {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "map a route to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "take me over to {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "get directions for {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "navigate me to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "sat nav to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "lead the way to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "i want to drive to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "head for {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "can you navigate to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "work out a route to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "show me how to get to {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "point me towards {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "get me over to {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "i need to drive to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "directions please to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "take me down to {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "get me home"
        weight: 0.6
        args:
          destination: "home"
      - text: "navigate home"
        weight: 0.95
        args:
          destination: "home"
      - text: "drive me home"
        weight: 0.95
        args:
          destination: "home"
      - text: "quickest way home"
        weight: 0.95
        args:
          destination: "home"
      - text: "i want to head home"
        weight: 0.75
        args:
          destination: "home"
      - text: "get me to work"
        weight: 0.75
        args:
          destination: "work"
      - text: "directions to work"
        weight: 0.95
        args:
          destination: "work"
      - text: "navigate to the office"
        weight: 0.95
        args:
          destination: "office"
      - text: "take me to my house"
        weight: 0.95
        args:
          destination: "my house"
      - text: "bring me home"
        weight: 0.75
        args:
          destination: "home"
      - text: "how do i get home from here"
        weight: 0.6
        args:
          destination: "home"
      - text: "sat nav home"
        weight: 0.95
        args:
          destination: "home"
      - text: "route me home"
        weight: 0.95
        args:
          destination: "home"
      - text: "get me back home"
        weight: 0.6
        args:
          destination: "home"
      - text: "directions to the {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "how do i get to the {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "show me the way to the {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "take me {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "fastest route {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "guide me to the {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "put the {destination} in my sat nav"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "i need to get to the {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "what is the best way to get to the {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "i want to drive over to my {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "can you map out a journey to {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
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
