---
name: github-zen
description: Recupera una frase di saggezza zen dall'API di GitHub. Usato per testare l'import host http_fetch end-to-end. Richiede accesso a internet.
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
        - keywords: [zen]
          weight: 0.9
        - keywords: [saggezza]
          weight: 0.9
        - keywords: [github, zen]
          weight: 0.95
    examples:
      - text: "zen di github"
      - text: "dimmi una frase di saggezza"
      - text: "dammi un po' di zen"
      - text: "dimmi qualcosa di saggio"
      - text: "condividi un po' di saggezza"
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# GitHub Zen (Italiano)

Dice uno degli aforismi "zen di GitHub" — come *"Speak like a human."* o *"Approachable is better than simple."* — recuperato in tempo reale da `https://api.github.com/zen`.
