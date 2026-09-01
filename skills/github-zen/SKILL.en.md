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
      - text: "github zen please"
        weight: 0.95
      - text: "give me some github zen"
        weight: 0.95
      - text: "hit me with some github zen"
        weight: 0.95
      - text: "show me the github zen"
        weight: 0.95
      - text: "whats the github zen"
        weight: 0.95
      - text: "fetch me a github zen"
        weight: 0.95
      - text: "github zen quote"
        weight: 0.95
      - text: "a bit of github zen please"
        weight: 0.95
      - text: "some zen please"
        weight: 0.95
      - text: "give me some zen"
        weight: 0.95
      - text: "hit me with some zen"
        weight: 0.95
      - text: "i could use some zen"
        weight: 0.95
      - text: "share some zen"
        weight: 0.95
      - text: "drop some zen on me"
        weight: 0.95
      - text: "a little zen please"
        weight: 0.95
      - text: "zen me"
        weight: 0.95
      - text: "give me a zen quote"
        weight: 0.95
      - text: "read me some zen"
        weight: 0.95
      - text: "whats todays zen"
        weight: 0.95
      - text: "any zen for me"
        weight: 0.95
      - text: "lay some zen on me"
        weight: 0.95
      - text: "i need some zen"
        weight: 0.95
      - text: "gimme zen"
        weight: 0.95
      - text: "tell me a zen saying"
        weight: 0.95
      - text: "some words of zen please"
        weight: 0.95
      - text: "a zen one liner please"
        weight: 0.95
      - text: "give me a piece of wisdom"
        weight: 0.95
      - text: "share a piece of wisdom"
        weight: 0.95
      - text: "hit me with some wisdom"
        weight: 0.95
      - text: "drop some wisdom on me"
        weight: 0.95
      - text: "give me some wisdom"
        weight: 0.95
      - text: "a bit of wisdom please"
        weight: 0.95
      - text: "lay some wisdom on me"
        weight: 0.95
      - text: "i need some wisdom"
        weight: 0.95
      - text: "some wisdom for the day"
        weight: 0.95
      - text: "give me a nugget of wisdom"
        weight: 0.95
      - text: "share a nugget of wisdom"
        weight: 0.95
      - text: "wisdom please"
        weight: 0.95
      - text: "got any wisdom for me"
        weight: 0.95
      - text: "any wisdom to share"
        weight: 0.95
      - text: "teach me something wise"
        weight: 0.95
      - text: "tell me something wise"
        weight: 0.85
      - text: "say something wise to me"
        weight: 0.85
      - text: "give me something wise"
        weight: 0.85
      - text: "a wise word please"
        weight: 0.95
      - text: "share a wise word"
        weight: 0.95
      - text: "give me a wise saying"
        weight: 0.85
      - text: "tell me a wise saying"
        weight: 0.85
      - text: "something wise please"
        weight: 0.95
      - text: "a wise thought please"
        weight: 0.95
      - text: "inspire me with some wisdom"
        weight: 0.95
      - text: "give me a wise one liner"
        weight: 0.85
      - text: "drop a wise line on me"
        weight: 0.95
      - text: "words of wisdom please"
        weight: 0.95
      - text: "give me your best wisdom"
        weight: 0.85
      - text: "some sage advice please"
        weight: 0.95
      - text: "share some sage wisdom"
        weight: 0.95
      - text: "hit me with a wise quote"
        weight: 0.95
      - text: "a wise quote please"
        weight: 0.95
      - text: "give me a wise quote"
        weight: 0.95
      - text: "read me a piece of wisdom"
        weight: 0.95
      - text: "i want some wisdom"
        weight: 0.95
      - text: "hand me some wisdom"
        weight: 0.95
      - text: "can i get some wisdom"
        weight: 0.95
      - text: "wisdom for the road please"
        weight: 0.95
      - text: "give me a zen thought"
        weight: 0.95
      - text: "a zen thought please"
        weight: 0.95
      - text: "share a zen thought"
        weight: 0.95
      - text: "some zen wisdom please"
        weight: 0.95
      - text: "give me some zen wisdom"
        weight: 0.95
      - text: "a bit of zen wisdom"
        weight: 0.95
      - text: "zen wisdom please"
        weight: 0.95
      - text: "bestow some wisdom on me"
        weight: 0.95
      - text: "enlighten me with some wisdom"
        weight: 0.95
      - text: "a little wisdom to start the day"
        weight: 0.95
      - text: "start me off with some wisdom"
        weight: 0.95
      - text: "a wise thought to ponder please"
        weight: 0.95
      - text: "share a wise thought for the day"
        weight: 0.95
      - text: "dish out some wisdom"
        weight: 0.95
      - text: "serve me some wisdom"
        weight: 0.95
      - text: "offer me some wisdom"
        weight: 0.95
      - text: "throw some wisdom my way"
        weight: 0.95
      - text: "send some wisdom my way"
        weight: 0.95
      - text: "a dose of wisdom please"
        weight: 0.95
      - text: "give me my daily wisdom"
        weight: 0.95
      - text: "daily zen please"
        weight: 0.95
      - text: "my daily dose of zen"
        weight: 0.95
      - text: "one more piece of wisdom"
        weight: 0.95
      - text: "give me a wise little saying"
        weight: 0.95
      - text: "give me a bit of github wisdom"
        weight: 0.95
      - text: "github zen"
        weight: 0.95
      - text: "tell me some wisdom"
        weight: 0.95
      - text: "give me a piece of zen"
        weight: 0.95
      - text: "say something wise"
        weight: 0.85
      - text: "share some wisdom with me"
        weight: 0.95
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
