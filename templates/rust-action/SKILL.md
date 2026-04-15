---
name: my-action-skill
description: Starter for a Rust skill that returns a structured Action response rather than plain text. Use when the frontend needs to launch an app, render a rich card, or mirror persisted state.
license: MIT
metadata:
  ari:
    id: com.example.myactionskill
    version: "0.1.0"
    author: Your Name
    engine: ">=0.3"
    capabilities: []
    languages: [en]
    specificity: high
    matching:
      patterns:
        - keywords: [my, action]
          weight: 0.95
      custom_score: false
    examples:
      - text: "my action"
      - text: "do my action"
      - text: "trigger my action"
      - text: "run my action"
      - text: "please my action"
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# My Action Skill

Emits a structured `Response::Action` envelope that the frontend interprets. See [../../docs/action-responses.md](../../docs/action-responses.md) for the envelope contract.

The template emits a 30-second countdown card with an attached critical alert as a starting point. Replace it with whatever primitives your skill needs — cards with progress, ongoing notifications, single-shot `launch_app` / `search` / `clipboard` slots, dismissals. The envelope is composed of named primitives, not discriminator-tagged variants; add fields, don't add types.

## Example utterances

- "my action"
