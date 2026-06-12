---
name: coin-flip
description: Lancia una moneta virtuale e restituisce testa o croce. Usalo quando l'utente chiede di lanciare una moneta, tirare una moneta o fare una scelta binaria casuale.
license: MIT
metadata:
  ari:
    id: dev.heyari.coinflip
    version: "0.1.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.1"
    capabilities: []
    languages: [en, it]
    specificity: high
    matching:
      patterns:
        - keywords: [lancia, moneta]
          weight: 0.95
        - keywords: [tira, moneta]
          weight: 0.95
        - keywords: [testa, croce]
          weight: 0.9
    declarative:
      response_pick: ["Heads.", "Tails."]
---

# Coin Flip (Italiano)

Lancia una moneta virtuale. Restituisce "Testa." o "Croce." in modo casuale.
