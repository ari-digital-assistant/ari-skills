---
name: counter
description: Counts how many times you've asked it to count, persisting across calls. Single-digit ASCII counter that wraps from 9 back to 1. Reference WASM skill for the storage_kv host imports.
license: MIT
metadata:
  ari:
    id: dev.heyari.counter
    version: "0.1.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.1"
    capabilities: [storage_kv]
    languages: [en]
    specificity: high
    matching:
      patterns:
        - keywords: [count]
          weight: 0.95
        - keywords: [tick]
          weight: 0.95
    examples:
      - text: "count"
      - text: "tick"
      - text: "increment the counter"
      - text: "count up"
      - text: "add one to the counter"
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# Counter

Reference WASM skill for the `ari::storage_get` and `ari::storage_set` host
imports. Each call increments a single ASCII digit stored under the key
`counter`. Persists across CLI invocations because the storage file lives on
disk.

## Why it exists

This is the simplest possible end-to-end test of the WASM ABI's storage
imports:

1. The skill manifest declares `[storage_kv]`.
2. The loader's install-time capability check confirms the host grants
   `storage_kv`.
3. The loader's import sneak guard confirms the module's `ari::storage_get`
   and `ari::storage_set` imports are matched by the manifest declaration.
4. On execute, the WASM module:
   - calls `storage_get("counter")` to read the existing count
   - if absent, writes "1"; otherwise increments the digit (wrapping `9 → 1`)
   - calls `storage_set("counter", "<new-digit>")` to persist
   - returns the new digit as the response text
5. The host opens the per-skill JSON file at `<storage_root>/dev.heyari.counter.json`,
   updates the `counter` key, and atomically renames the temp file into place.

## Try it

```
ari-cli --host-capabilities=storage_kv,notifications,launch_app,clipboard,tts \
        --storage-dir /tmp/ari-counter-demo \
        --extra-skill-dir ../ari-skills/skills/counter \
        "count"
```

Run it three times in a row. The output goes `1`, `2`, `3`. Look at
`/tmp/ari-counter-demo/dev.heyari.counter.json` between calls — it'll contain
`{"counter":"2"}`, then `{"counter":"3"}`.

## Caveats

- Requires `--host-capabilities=storage_kv`. The default `pure_frontend` host
  won't grant it.
- Requires a writable `--storage-dir`. The CLI defaults to a system-temp
  directory if you don't supply one, which is fine for sideloading but won't
  survive reboots.
- Single-digit only — it wraps from 9 back to 1. A real skill would format
  multi-digit integers, but the WAT for that is gnarly. Once a Rust→wasm32
  toolchain is part of the workflow, this'll be the first skill to get
  rewritten properly.
