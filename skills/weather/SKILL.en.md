---
name: weather
description: Current weather, forecasts, and conditions like wind, rain, and UV — for your current location or any place you name.
license: MIT
metadata:
  ari:
    id: dev.heyari.weather
    version: "0.1.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [http, location, storage_kv]
    languages: [en, it]
    specificity: high
    matching:
      # Patterns are matched against POST-NORMALISED input: lowercased,
      # with apostrophes/contractions expanded ("how's" → "how is",
      # "what's" → "what is") BEFORE the engine runs the regex. So the
      # patterns below stay lowercase and apostrophe-free.
      patterns:
        - regex: "\\bweather\\b"
          weight: 0.95
        - regex: "\\bforecast\\b"
          weight: 0.9
        - regex: "\\b(will it|is it going to) rain\\b"
          weight: 0.9
        - regex: "\\b(wind|windy)\\b"
          weight: 0.75
        - regex: "\\buv( index)?\\b"
          weight: 0.8
      custom_score: false
    # Examples carry `args` so FunctionGemma learns to extract the two
    # slots the skill needs: `location` (empty string = use GPS) and
    # `when` (one of now | today | tomorrow | this week).
    examples:
      - text: "how is the weather"
        args:
          location: ""
          when: "now"
      - text: "what is the weather in tokyo"
        args:
          location: "tokyo"
          when: "now"
      - text: "weather in valletta tomorrow"
        args:
          location: "valletta"
          when: "tomorrow"
      - text: "what is the forecast this week"
        args:
          location: ""
          when: "this week"
      - text: "will it rain today"
        args:
          location: ""
          when: "today"
      - text: "is it windy"
        args:
          location: ""
          when: "now"
      - text: "what is the uv index"
        args:
          location: ""
          when: "now"
      # Oblique phrasings the keyword patterns above deliberately miss —
      # these are the ones the router actually sees in production.
      - text: "will i need a coat later"
        args:
          location: ""
          when: "today"
      - text: "how hot is it outside"
        args:
          location: ""
          when: "now"
      - text: "should i take an umbrella tomorrow"
        args:
          location: ""
          when: "tomorrow"
      - text: "is it going to be cold in london tomorrow"
        args:
          location: "london"
          when: "tomorrow"
      - text: "do i need sunscreen today"
        args:
          location: ""
          when: "today"
      - text: "is the sun out in valletta"
        args:
          location: "valletta"
          when: "now"
      - text: "any chance of snow this week"
        args:
          location: ""
          when: "this week"
      - text: "will it be chilly tomorrow morning"
        args:
          location: ""
          when: "tomorrow"
    settings:
      - key: units
        label: Units
        type: select
        default: auto
        options:
          - value: auto
            label: Automatic
          - value: metric
            label: Metric (°C, km/h)
          - value: imperial
            label: Imperial (°F, mph)
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Weather

Current conditions and forecasts, plus facet queries like wind, rain
chance, and UV index. Ask about the weather where you are — the skill
uses a coarse device location — or name any place ("weather in tokyo").

## Supported utterances

- `how is the weather` — current conditions, current location
- `what is the weather in tokyo` — current conditions, named place
- `weather in valletta tomorrow` — named place, next day
- `what is the forecast this week` — multi-day outlook
- `will it rain today` — precipitation facet
- `is it windy` — wind facet
- `what is the uv index` — UV facet

## Extracted arguments

The router extracts two slots:

- `location` — the place name, or an empty string to use the device's
  coarse location via the `location` capability.
- `when` — one of `now`, `today`, `tomorrow`, or `this week`. Defaults
  to `now` when the utterance carries no time phrase.

## Settings

- **Units** — `Automatic` (follows the device locale), `Metric`
  (°C, km/h), or `Imperial` (°F, mph).

## Backend

Forecasts are sourced over the `http` capability from keyless providers
(Open-Meteo), with results cached briefly via `storage_kv` to avoid
hammering the API on repeat asks.
