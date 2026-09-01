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
      - text: "dammi una frase di saggezza"
        weight: 0.95
      - text: "dammi una perla di saggezza"
        weight: 0.95
      - text: "dammi un aforisma zen"
        weight: 0.95
      - text: "dammi una massima saggia"
        weight: 0.95
      - text: "dammi un pensiero saggio"
        weight: 0.95
      - text: "dammi una citazione zen"
        weight: 0.95
      - text: "dammi un consiglio saggio"
        weight: 0.95
      - text: "dimmi un po' di zen"
        weight: 0.85
      - text: "dimmi una perla di saggezza"
        weight: 0.95
      - text: "dimmi un aforisma zen"
        weight: 0.95
      - text: "dimmi una massima saggia"
        weight: 0.95
      - text: "dimmi un pensiero saggio"
        weight: 0.95
      - text: "dimmi una citazione zen"
        weight: 0.95
      - text: "dimmi un consiglio saggio"
        weight: 0.95
      - text: "regalami un po' di zen"
        weight: 0.95
      - text: "regalami una frase di saggezza"
        weight: 0.95
      - text: "regalami una perla di saggezza"
        weight: 0.95
      - text: "regalami un aforisma zen"
        weight: 0.95
      - text: "regalami una massima saggia"
        weight: 0.95
      - text: "regalami un pensiero saggio"
        weight: 0.95
      - text: "regalami una citazione zen"
        weight: 0.95
      - text: "regalami un consiglio saggio"
        weight: 0.95
      - text: "condividi un po' di zen"
        weight: 0.95
      - text: "condividi una frase di saggezza"
        weight: 0.95
      - text: "condividi una perla di saggezza"
        weight: 0.95
      - text: "condividi un aforisma zen"
        weight: 0.95
      - text: "condividi una massima saggia"
        weight: 0.95
      - text: "condividi un pensiero saggio"
        weight: 0.95
      - text: "condividi una citazione zen"
        weight: 0.95
      - text: "condividi un consiglio saggio"
        weight: 0.95
      - text: "mostrami un po' di zen"
        weight: 0.85
      - text: "mostrami una frase di saggezza"
        weight: 0.95
      - text: "mostrami una perla di saggezza"
        weight: 0.95
      - text: "mostrami un aforisma zen"
        weight: 0.95
      - text: "mostrami una massima saggia"
        weight: 0.95
      - text: "mostrami un pensiero saggio"
        weight: 0.95
      - text: "mostrami una citazione zen"
        weight: 0.95
      - text: "mostrami un consiglio saggio"
        weight: 0.95
      - text: "voglio un po' di zen"
        weight: 0.85
      - text: "voglio una frase di saggezza"
        weight: 0.95
      - text: "voglio una perla di saggezza"
        weight: 0.95
      - text: "voglio un aforisma zen"
        weight: 0.95
      - text: "voglio una massima saggia"
        weight: 0.95
      - text: "voglio un pensiero saggio"
        weight: 0.95
      - text: "voglio una citazione zen"
        weight: 0.95
      - text: "voglio un consiglio saggio"
        weight: 0.95
      - text: "offrimi un po' di zen"
        weight: 0.95
      - text: "offrimi una frase di saggezza"
        weight: 0.95
      - text: "offrimi una perla di saggezza"
        weight: 0.95
      - text: "offrimi un aforisma zen"
        weight: 0.95
      - text: "offrimi una massima saggia"
        weight: 0.95
      - text: "offrimi un pensiero saggio"
        weight: 0.95
      - text: "offrimi una citazione zen"
        weight: 0.95
      - text: "offrimi un consiglio saggio"
        weight: 0.95
      - text: "raccontami un po' di zen"
        weight: 0.95
      - text: "raccontami una frase di saggezza"
        weight: 0.95
      - text: "raccontami una perla di saggezza"
        weight: 0.95
      - text: "raccontami un aforisma zen"
        weight: 0.95
      - text: "raccontami una massima saggia"
        weight: 0.95
      - text: "raccontami un pensiero saggio"
        weight: 0.95
      - text: "raccontami una citazione zen"
        weight: 0.95
      - text: "raccontami un consiglio saggio"
        weight: 0.95
      - text: "qual è lo zen di github"
        weight: 0.95
      - text: "dammi lo zen di github"
        weight: 0.95
      - text: "condividi lo zen di github"
        weight: 0.95
      - text: "zen del giorno"
        weight: 0.95
      - text: "dammi lo zen del giorno"
        weight: 0.85
      - text: "frase zen del giorno"
        weight: 0.95
      - text: "una frase zen"
        weight: 0.95
      - text: "dimmi qualcosa di profondo"
        weight: 0.85
      - text: "dimmi qualcosa di zen"
        weight: 0.85
      - text: "illuminami con un po' di zen"
        weight: 0.95
      - text: "fammi riflettere con una massima"
        weight: 0.95
      - text: "un pensiero profondo per favore"
        weight: 0.95
      - text: "ho bisogno di un po' di zen"
        weight: 0.85
      - text: "ho bisogno di saggezza"
        weight: 0.95
      - text: "voglio un po' di saggezza"
        weight: 0.85
      - text: "un po' di zen grazie"
        weight: 0.85
      - text: "una citazione di github zen"
        weight: 0.95
      - text: "aforisma zen per favore"
        weight: 0.95
      - text: "dammi una massima"
        weight: 0.95
      - text: "dimmi una frase saggia"
        weight: 0.95
      - text: "un consiglio zen"
        weight: 0.95
      - text: "una perla zen"
        weight: 0.95
      - text: "dammi ispirazione zen"
        weight: 0.95
      - text: "qualche parola saggia"
        weight: 0.95
      - text: "regalami una riflessione"
        weight: 0.95
      - text: "dimmi un proverbio saggio"
        weight: 0.95
      - text: "dammi una perla zen"
        weight: 0.95
      - text: "dimmi due parole di saggezza"
        weight: 0.95
      - text: "zen di github"
        weight: 0.95
      - text: "dimmi una frase di saggezza"
        weight: 0.95
      - text: "dammi un po' di zen"
        weight: 0.85
      - text: "dimmi qualcosa di saggio"
        weight: 0.85
      - text: "condividi un po' di saggezza"
        weight: 0.95
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# GitHub Zen (Italiano)

Dice uno degli aforismi "zen di GitHub" — come *"Speak like a human."* o *"Approachable is better than simple."* — recuperato in tempo reale da `https://api.github.com/zen`.
