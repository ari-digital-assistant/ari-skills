---
name: timer
description: Sets, queries, and cancels named timers. Supports natural phrasing like "set a pasta timer for 8 minutes" or "set a 4 minute pasta timer". Handles multiple simultaneous timers.
license: MIT
metadata:
  ari:
    id: dev.heyari.timer
    version: "0.1.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [storage_kv]
    languages: [en]
    specificity: high
    matching:
      patterns:
        - regex: "\\b(set|start|create|add)\\b.*\\btimer\\b"
          weight: 0.95
        - regex: "\\btimer\\b.*\\b(for|of)\\b"
          weight: 0.9
        - regex: "\\b(how much|how long|time left|time remaining|how many)\\b.*\\btimer\\b"
          weight: 0.95
        - regex: "\\b(cancel|stop|remove|delete|clear)\\b.*\\btimer\\b"
          weight: 0.95
        - regex: "\\bwhat timers\\b|\\blist.*timer|\\btimers do i\\b"
          weight: 0.9
      custom_score: false
    examples:
      - text: "set a timer for 10 minutes"
      - text: "set a pasta timer for 8 minutes"
      - text: "set a 4 minute pasta timer"
      - text: "how much time is left on my pasta timer"
      - text: "cancel my pasta timer"
      - text: "what timers do i have"
      - text: "set a timer for 5 minutes and another for 15 minutes"
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Timer

Sets and tracks named timers.

## Supported utterances

- `set a timer for 10 minutes` — anonymous timer
- `set a pasta timer for 8 minutes` — named timer (prepositional form)
- `set a 4 minute pasta timer` — named timer (adjective form)
- `set a timer for 5 minutes and another for 15 minutes` — multi-create
- `how much time is left on my pasta timer` — query
- `cancel my pasta timer` / `stop my pasta timer` — cancel
- `what timers do I have` / `list my timers` — list

## Notes

Timer state is persisted under this skill's `storage_kv` file. Expired timers
are pruned on every invocation, so orphaned entries from a background-killed
app self-heal.

This skill returns `Response::Action` payloads with an envelope the frontend
can use to render a live countdown card and/or schedule an expiry alarm. See
[docs/action-responses.md](../../docs/action-responses.md) for the schema.
