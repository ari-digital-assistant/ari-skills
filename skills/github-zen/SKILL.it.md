---
name: github-zen
description: Recupera una frase di saggezza zen dall'API di GitHub. Usato per testare l'import host http_fetch end-to-end. Richiede accesso a internet.
license: MIT
metadata:
  ari:
    id: dev.heyari.githubzen
    version: "0.1.1"
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
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# GitHub Zen (Italiano)

Skill WASM di riferimento che esercita l'import host `ari::http_fetch`. All'esecuzione, il modulo chiama `http_fetch` con l'URL fisso `https://api.github.com/zen` e restituisce l'envelope JSON come testo di risposta.

Il campo body della risposta è uno dei aforismi "zen di GitHub", come *"Speak like a human."* o *"Approachable is better than simple."*.
