---
name: weather
description: Current weather, forecasts, and conditions like wind, rain, and UV.
license: MIT
metadata:
  ari:
    id: dev.heyari.weather
    version: "0.1.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [http, location, storage_kv]
    languages: [en]
    specificity: high
    matching:
      patterns:
        - regex: "\\bweather\\b"
          weight: 0.95
      custom_score: false
    examples:
      - text: "how is the weather"
      - text: "what is the weather today"
      - text: "will it rain tomorrow"
      - text: "weather forecast for the weekend"
      - text: "is it windy outside"
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Weather

Current conditions, forecasts, and weather facets via MET Norway and Open-Meteo.
