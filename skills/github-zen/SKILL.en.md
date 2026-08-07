---
name: github-zen
description: Fetches a one-line piece of zen wisdom from the GitHub API. Use to test the WASM http_fetch host import end-to-end. Requires internet access.
license: MIT
metadata:
  ari:
    id: dev.heyari.githubzen
    version: "0.1.2"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.1"
    capabilities: [http]
    languages: [en, it]
    specificity: high
    matching:
      patterns:
        # Either word on its own is enough — "tell me a zen", "some
        # wisdom please", etc. all have to land on this skill.
        - keywords: [zen]
          weight: 0.9
        - keywords: [wisdom]
          weight: 0.9
        # If both words appear, or "github" + "zen", prefer this skill
        # over anything else that also matches a single word.
        - keywords: [github, zen]
          weight: 0.95
    examples:
      - text: "github zen"
      - text: "tell me some wisdom"
      - text: "give me a piece of zen"
      - text: "say something wise"
      - text: "share some wisdom with me"
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# GitHub Zen

Speaks one of GitHub's "zen of GitHub" one-liners — *"Speak like a human."*,
*"Approachable is better than simple."* — fetched live from
`https://api.github.com/zen`.

## Why it exists

It's the simplest end-to-end exercise of the WASM ABI's `http_fetch` import:

1. The skill manifest declares `[http]`.
2. The loader's install-time capability check confirms the host grants `http`.
3. The loader's import sneak guard confirms the module's `ari::http_fetch`
   import is matched by the manifest declaration.
4. On execute, the WASM module calls `http_fetch` with the URL pointer.
5. The host fires a real HTTPS GET (TLS via rustls), reads the body, encodes
   `{"status": 200, "body": "..."}` JSON, allocates space in the skill's
   linear memory via `ari_alloc`, copies the JSON in, and returns the packed
   pointer.
6. The skill reads the body out of that envelope and speaks it. Until 0.1.2 it
   returned the envelope verbatim, which meant users heard raw JSON.

## Example utterances

- "github zen"
- "tell me some wisdom"

## Caveats

- Requires `--host-capabilities=http` (or any capability set including http)
  on the CLI. The default `pure_frontend` host won't grant http.
- Requires internet access at call time. Anything other than a 2xx with a
  body — network down, GitHub having a bad day — is logged and answered with
  the `unavailable` string.
