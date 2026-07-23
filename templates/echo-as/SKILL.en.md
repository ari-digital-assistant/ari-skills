---
name: echo-as
description: Repeats back whatever the user said, as an AssemblyScript starter skill. Use when the user asks the assistant to echo something, repeat after them, or say their words back.
license: MIT
metadata:
  ari:
    id: com.example.echoas
    version: "0.1.0"
    author: Your Name <you@example.com>
    engine: ">=0.3,<0.4"
    capabilities: []
    languages: [en]
    specificity: high
    matching:
      patterns:
        - keywords: [echo, this]
          weight: 0.95
        - keywords: [repeat, after, me]
          weight: 0.95
      custom_score: false
    examples:
      - text: "say that back to me"
      - text: "parrot what i just said"
      - text: "can you say it again for me"
      - text: "mirror my words"
      - text: "read that back"
      - text: "just say what i say"
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Echo (AssemblyScript)

A starter skill for authors who can't use Rust.

**Read [assemblyscript.md](../../docs/assemblyscript.md) before choosing this
template.** The AssemblyScript SDK covers input, responses, logging,
capability checks, HTTP and key-value storage — and nothing else. There is no
typed envelope builder, no settings helpers, no OAuth, no i18n, and no
wrapper for location, tasks, calendar or media. Rust is the supported path;
this exists so the ABI isn't Rust-only.

## Using this template

1. Copy the directory and rename it. Directory name and `name:` must match.
2. Change `metadata.ari.id` to a reverse-DNS id under a domain you control.
3. Edit `assembly/index.ts`. Keep the `export { ari_alloc }` line — the host
   needs it to write your input into memory.
4. `./build.sh`, then validate.

The build must pass `--use abort=`; the host provides no `env::abort`, and a
module importing it fails to instantiate. The bundled `build.sh` handles this.
