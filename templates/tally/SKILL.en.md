---
name: tally
description: Keeps a running count of something the user is tracking through the day. Use when the user wants to add to a tally, log another one, check how many they have counted so far, or reset the count.
license: MIT
metadata:
  ari:
    id: com.example.tally
    version: "0.1.0"
    author: Your Name <you@example.com>
    homepage: https://example.com/tally
    engine: ">=0.3,<0.4"
    capabilities: [storage_kv]
    languages: [en]
    specificity: high
    matching:
      patterns:
        - keywords: [add, tally]
          weight: 0.95
        - keywords: [my, tally]
          weight: 0.95
        - keywords: [reset, tally]
          weight: 0.95
    examples:
      - text: "that is another one for today"
      - text: "log another one"
      - text: "how many have i had so far"
      - text: "chalk one up"
      - text: "put me down for one more"
      - text: "start me back at zero"
      - text: "wipe the count and start again"
      - text: "where am i up to today"
    settings:
      - key: label
        label: "What are you counting?"
        type: text
        required: false
        default: "things"
        help_text: "Shown on the card, e.g. cups of water."
      - key: goal
        label: "Daily goal"
        type: select
        required: false
        default: "0"
        options:
          - value: "0"
            label: "No goal"
          - value: "5"
            label: "5 a day"
          - value: "8"
            label: "8 a day"
          - value: "10"
            label: "10 a day"
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Tally

Keeps a running count in per-skill storage.

- "add one to my tally" → increments, replies with the new total
- "what is my tally" → reports the current total
- "reset my tally" → back to zero

## What this template demonstrates

| Thing | Where |
|---|---|
| A capability-gated host import | `capabilities: [storage_kv]` + `features = ["storage"]` |
| Reading user settings | `ari::setting_get("label")` |
| Branching on the utterance inside `execute` | `src/lib.rs` |
| Emitting a rich card | `presentation::Stat` |
| Localisable output | every string goes through `ari::t` |

Note that all three patterns select the *same* skill. Patterns only decide
which skill runs; `execute` re-reads the utterance and decides what to do.

## Using this template

1. Copy the directory and rename it. The directory name and the `name:` field
   must match.
2. Change `metadata.ari.id` to a reverse-DNS id under a domain you control.
3. Rewrite `description`, the patterns, the examples and `strings/en.json`.
4. Drop `capabilities` and the `storage` SDK feature if you don't need them —
   declare exactly what you use, no more.
5. `./build.sh`, then `./tools/validate templates/tally` (with your path).

Full walkthrough: [tutorial-wasm.md](../../docs/tutorial-wasm.md).
