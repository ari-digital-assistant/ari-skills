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
    # Gli esempi alimentano il router FunctionGemma, che entra in gioco
    # SOLO quando lo scorer a keyword non trova nulla. Le frasi che i
    # pattern qui sopra non catturano ("lancio della moneta", "decida il
    # caso") sono quindi le più utili: sono esattamente i casi per cui il
    # router esiste.
    examples:
      - text: "lancia una moneta"
      - text: "tira una moneta"
      - text: "testa o croce"
      - text: "lancio della moneta"
      - text: "puoi lanciare una moneta per me"
      - text: "tira una moneta per favore"
      - text: "facciamo a testa o croce"
      - text: "mi serve un testa o croce"
      - text: "testa o croce per favore"
      - text: "lasciamo decidere al caso"
      - text: "lancio di una monetina"
      - text: "fai testa o croce"
      - text: "scegli testa o croce"
      - text: "che decida il caso"
      - text: "aiutami a decidere con una moneta"
    declarative:
      response_pick: ["coinflip.heads", "coinflip.tails"]
---

# Coin Flip (Italiano)

Lancia una moneta virtuale. Restituisce "Testa." o "Croce." in modo casuale.
