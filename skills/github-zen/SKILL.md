---
name: github-zen
description: Fetches a one-line piece of zen wisdom from the GitHub API. Use to test the WASM http_fetch host import end-to-end. Requires internet access.
license: MIT
metadata:
  ari:
    id: dev.heyari.githubzen
    version: "0.1.1"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3,<0.4"
    capabilities: [http]
    languages: [en]
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
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# GitHub Zen

Reference WASM skill that exercises the `ari::http_fetch` host import. On
`execute`, the module calls `http_fetch` with a hardcoded `https://api.github.com/zen`
URL and returns the JSON envelope verbatim as the response text.

The body field of the response is one of GitHub's "zen of GitHub" one-liners,
like *"Speak like a human."* or *"Approachable is better than simple."*.

## Why it exists

This skill is the simplest possible end-to-end test of the WASM ABI's
`http_fetch` import:

1. The skill manifest declares `[http]`.
2. The loader's install-time capability check confirms the host grants `http`.
3. The loader's import sneak guard confirms the module's `ari::http_fetch`
   import is matched by the manifest declaration.
4. On execute, the WASM module calls `http_fetch` with the URL pointer.
5. The host fires a real HTTPS GET (TLS via rustls), reads the body, encodes
   `{"status": 200, "body": "..."}` JSON, allocates space in the skill's
   linear memory via `ari_alloc`, copies the JSON in, and returns the packed
   pointer.
6. The skill's `execute` function does nothing more than return that same
   packed pointer back to the host. The host reads the JSON out and emits it
   as the response text.

## Example utterances

- "github zen"
- "tell me some wisdom"

## Caveats

- Requires `--host-capabilities=http` (or any capability set including http)
  on the CLI. The default `pure_frontend` host won't grant http.
- Requires internet access at call time. If the network's down, the response
  will be `{"status": 0, "body": null, "error": "request failed: ..."}`.
- The output is the raw JSON envelope, not the extracted body. A real skill
  would parse the JSON inside the WASM and return just the body string. We
  keep this skill stupid simple to focus on the ABI plumbing.
