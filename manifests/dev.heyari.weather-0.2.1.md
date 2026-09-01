---
name: weather
description: Current weather, forecasts, and conditions like rain chance, wind, humidity, and UV — for your current location or any place you name.
license: MIT
metadata:
  ari:
    id: dev.heyari.weather
    version: "0.2.1"
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
        - regex: "\\bhumid(ity)?\\b"
          weight: 0.8
      custom_score: false
    # Examples carry `args` so FunctionGemma learns to extract the two
    # slots the skill needs: `location` (empty string = use GPS) and
    # `when` (one of now | today | tomorrow | this week).
    examples:
      - text: "what are the weather conditions {location} right now"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "give me the current weather {location}"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "how warm is it {location} at the moment"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "what is the temperature {location} right now"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "what does it feel like outside {location} right now"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "is any rain falling {location} right now"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "how strong is the wind {location} at the moment"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "what is the humidity {location} right now"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "how high is the uv level {location} right now"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "are the skies clear {location} at the moment"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "how is the weather shaping up {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "give me today's forecast {location}"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "is rain expected {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "are there showers coming later {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "will it be windy {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "what will the high temperature be {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "is it going to stay cold {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "will sunscreen be necessary {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "do I need an umbrella {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "how humid will it be {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "when will the uv be strongest {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "is there a chance of snow {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "is the weather suitable for being outdoors {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "show me tomorrow's forecast {location}"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "what will the weather be like {location} tomorrow"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "is rain likely {location} tomorrow"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "what temperature will it reach {location} tomorrow"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "will the wind pick up {location} tomorrow"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "could it snow {location} tomorrow"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "will I need a warm coat {location} tomorrow"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "should I bring an umbrella {location} tomorrow"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "will I need sun protection {location} tomorrow"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "is it expected to be humid {location} tomorrow"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "are clear skies expected {location} tomorrow"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "what is the weather outlook {location} this week"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "give me the weeklong forecast {location}"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "will it rain at any point {location} this week"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "what temperatures are expected {location} this week"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "are strong winds expected {location} this week"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "is snow in the forecast {location} this week"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "which days should be sunniest {location} this week"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "am I likely to need an umbrella {location} this week"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "will the weather be good for outdoor plans {location} this week"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "I'd like a quick weather update {location}"
        weight: 0.85
        args:
          location: "{location}"
          when: "now"
      - text: "can you check the current conditions {location}"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "tell me whether it is dry {location} right now"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "is there much cloud cover {location} at the moment"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "what does the weather feel like {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "I need to know today's weather {location}"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "let me know if thunderstorms are likely {location} today"
        weight: 0.95
        args:
          location: "{location}"
          when: "today"
      - text: "could you check tomorrow's weather {location}"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "I am curious whether tomorrow will be warmer {location}"
        weight: 0.95
        args:
          location: "{location}"
          when: "tomorrow"
      - text: "give me the rain outlook {location} for this week"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "which day has the best weather {location} this week"
        weight: 0.85
        args:
          location: "{location}"
          when: "this week"
      - text: "will freezing conditions be a concern {location} this week"
        weight: 0.95
        args:
          location: "{location}"
          when: "this week"
      - text: "how is the weather"
        weight: 0.95
        args:
          location: ""
          when: "now"
      - text: "what is the weather in {location}"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "weather in {location} {when}"
        weight: 0.95
        args:
          location: "{location}"
          when: "{when}"
      - text: "what is the forecast {when}"
        weight: 0.95
        args:
          location: ""
          when: "{when}"
      - text: "will it rain {when}"
        weight: 0.95
        args:
          location: ""
          when: "{when}"
      - text: "is it windy"
        weight: 0.95
        args:
          location: ""
          when: "now"
      - text: "what is the uv index"
        weight: 0.95
        args:
          location: ""
          when: "now"
      - text: "is it humid {when}"
        weight: 0.95
        args:
          location: ""
          when: "{when}"
      - text: "will i need a coat later"
        weight: 0.95
        args:
          location: ""
          when: "today"
      - text: "how hot is it outside"
        weight: 0.95
        args:
          location: ""
          when: "now"
      - text: "should i take an umbrella {when}"
        weight: 0.95
        args:
          location: ""
          when: "{when}"
      - text: "is it going to be cold in {location} {when}"
        weight: 0.95
        args:
          location: "{location}"
          when: "{when}"
      - text: "do i need sunscreen {when}"
        weight: 0.95
        args:
          location: ""
          when: "{when}"
      - text: "is the sun out in {location}"
        weight: 0.95
        args:
          location: "{location}"
          when: "now"
      - text: "any chance of snow {when}"
        weight: 0.95
        args:
          location: ""
          when: "{when}"
      - text: "will it be chilly {when} morning"
        weight: 0.6
        args:
          location: ""
          when: "{when}"
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
chance, humidity, and UV index. Ask about the weather where you are — the
skill uses a coarse device location — or name any place ("weather in
tokyo").

Current conditions are answered with a lead sentence plus whatever detail
earns its place: the day's remaining rain chance (and the hour it peaks,
when the rain clusters rather than drizzling all day), wind bearing and
speed, and humidity. A calm, dry day just gets the lead sentence.

## Supported utterances

- `how is the weather` — current conditions, current location
- `what is the weather in tokyo` — current conditions, named place
- `weather in valletta tomorrow` — named place, next day
- `what is the forecast this week` — multi-day outlook
- `will it rain today` — precipitation facet
- `is it windy` — wind facet, answered for the day asked about
- `what is the uv index` — UV facet
- `is it humid today` — humidity facet

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
