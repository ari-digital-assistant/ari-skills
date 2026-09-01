---
name: timer
description: Sets, queries, and cancels named timers. Supports natural phrasing like "set a pasta timer for 8 minutes" or "set a 4 minute pasta timer". Handles multiple simultaneous timers.
license: MIT
metadata:
  ari:
    id: dev.heyari.timer
    version: "0.2.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [storage_kv, critical_alert]
    languages: [en, it]
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
      - text: "set a timer for {minutes} minutes"
        weight: 0.95
      - text: "set a {minutes} minute timer"
        weight: 0.95
      - text: "set a {minutes} minute {name} timer"
        weight: 0.95
      - text: "set a {name} timer for {minutes} minutes"
        weight: 0.95
      - text: "start a timer for {minutes} minutes"
        weight: 0.95
      - text: "start a {minutes} minute timer"
        weight: 0.95
      - text: "put a timer on for {minutes} minutes"
        weight: 0.95
      - text: "give me a timer for {minutes} minutes"
        weight: 0.95
      - text: "timer for {minutes} minutes"
        weight: 0.95
      - text: "can you set a timer for {minutes} minutes"
        weight: 0.95
      - text: "set a timer for {hours} hours"
        weight: 0.95
      - text: "set a {hours} hour timer"
        weight: 0.95
      - text: "set a timer for {hours} hours and {minutes} minutes"
        weight: 0.95
      - text: "give me {minutes} minutes for the {name}"
        weight: 0.95
      - text: "{minutes} minute countdown for the {name}"
        weight: 0.95
      - text: "count down {minutes} minutes for me"
        weight: 0.95
      - text: "i need a {minutes} minute countdown"
        weight: 0.95
      - text: "countdown from {minutes} minutes"
        weight: 0.95
      - text: "count down from {minutes} minutes for the {name}"
        weight: 0.95
      - text: "let me know when {minutes} minutes are up"
        weight: 0.85
      - text: "tell me when {minutes} minutes have passed"
        weight: 0.95
      - text: "i want a countdown of {minutes} minutes"
        weight: 0.95
      - text: "kick off a {minutes} minute timer for {name}"
        weight: 0.95
      - text: "set the {name} timer for {minutes} minutes"
        weight: 0.95
      - text: "set a timer for the {name} for {minutes} minutes"
        weight: 0.95
      - text: "put a {minutes} minute timer on for the {name}"
        weight: 0.95
      - text: "set a timer for {minutes} minutes and another for {minutes2} minutes"
        weight: 0.95
      - text: "set one timer for {minutes} minutes and another for {minutes2}"
        weight: 0.85
      - text: "i need two timers {minutes} minutes and {minutes2} minutes"
        weight: 0.95
      - text: "how much time is left on my {name} timer"
        weight: 0.85
      - text: "how long left on the {name} timer"
        weight: 0.95
      - text: "hows my {name} timer doing"
        weight: 0.95
      - text: "time left on the {name} timer"
        weight: 0.95
      - text: "how long is left on the {name} timer"
        weight: 0.95
      - text: "is the {name} timer done yet"
        weight: 0.95
      - text: "how much longer on the {name} timer"
        weight: 0.95
      - text: "whats left on my {name} timer"
        weight: 0.95
      - text: "how many minutes left on the {name} timer"
        weight: 0.95
      - text: "what timers do i have"
        weight: 0.95
      - text: "what timers have i got running"
        weight: 0.95
      - text: "list my timers"
        weight: 0.95
      - text: "show me my timers"
        weight: 0.95
      - text: "how many timers do i have going"
        weight: 0.85
      - text: "cancel my {name} timer"
        weight: 0.95
      - text: "stop the {name} timer"
        weight: 0.95
      - text: "scrap the {name} timer"
        weight: 0.95
      - text: "delete the {name} timer"
        weight: 0.95
      - text: "clear the {name} timer"
        weight: 0.95
      - text: "cancel the timer"
        weight: 0.95
      - text: "get rid of the {name} timer"
        weight: 0.95
      - text: "turn off the {name} timer"
        weight: 0.95
      - text: "remove the {minutes} minute timer"
        weight: 0.95
      - text: "reset the {name} timer"
        weight: 0.95
      - text: "set a timer for 10 minutes"
        weight: 0.95
      - text: "set a pasta timer for 8 minutes"
        weight: 0.95
      - text: "set a 4 minute pasta timer"
        weight: 0.95
      - text: "how much time is left on my pasta timer"
        weight: 0.85
      - text: "cancel my pasta timer"
        weight: 0.95
      - text: "set a timer for 5 minutes and another for 15 minutes"
        weight: 0.95
      - text: "give me 8 minutes for pasta"
        weight: 0.85
      - text: "10 minute countdown for the bread"
        weight: 0.95
      - text: "kick off a 4 minute timer for tea"
        weight: 0.95
      - text: "I need a 5 minute countdown"
        weight: 0.95
      - text: "count down 12 minutes for me"
        weight: 0.95
      - text: "how long left on the pasta timer"
        weight: 0.95
      - text: "scrap the bread timer"
        weight: 0.95
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
[docs/reference-actions.md](../../docs/reference-actions.md) for the schema.
