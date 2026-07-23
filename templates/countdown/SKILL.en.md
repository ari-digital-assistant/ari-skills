---
name: countdown
description: Starts a thirty-second countdown that rings when it finishes. Use when the user wants a short countdown demo, or as a starting point for any skill that shows a live card and then raises an alert.
license: MIT
metadata:
  ari:
    id: com.example.countdown
    version: "0.1.0"
    author: Your Name <you@example.com>
    engine: ">=0.3,<0.4"
    capabilities: [critical_alert]
    languages: [en]
    specificity: high
    matching:
      patterns:
        - keywords: [countdown, demo]
          weight: 0.95
        - keywords: [start, countdown]
          weight: 0.95
      custom_score: false
    examples:
      - text: "show me what a countdown looks like"
      - text: "give me thirty seconds on the clock"
      - text: "demo the card thing"
      - text: "run the countdown example"
      - text: "i want to see the alert demo"
      - text: "put a ticking card on screen"
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Countdown

Emits an action envelope carrying a live countdown card with an
`on_complete` alert. The card ticks down in the chat; when it hits zero the
frontend fires the alert. Tapping Cancel sends "stop countdown" back through
the engine as an ordinary utterance.

Use it as the starting point for any skill whose output is a card, an alert
or a notification rather than a sentence. Full envelope contract:
[reference-actions.md](../../docs/reference-actions.md).

## Why `capabilities: [critical_alert]`

The alert asks for `full_takeover`, which is gated. A skill emitting only a
plain card needs no capability at all — drop the declaration if you drop the
alert.

## Using this template

1. Copy the directory and rename it. Directory name and `name:` must match.
2. Change `metadata.ari.id` to a reverse-DNS id under a domain you control.
3. Replace the envelope in `src/lib.rs` with your own primitives.
4. `./build.sh`, then validate.
