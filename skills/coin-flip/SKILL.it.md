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
      - text: "lancia la moneta"
        weight: 0.95
      - text: "tira la moneta"
        weight: 0.95
      - text: "tira una monetina"
        weight: 0.95
      - text: "tira la monetina"
        weight: 0.95
      - text: "gira una moneta"
        weight: 0.95
      - text: "gira la moneta"
        weight: 0.95
      - text: "gira una monetina"
        weight: 0.95
      - text: "gira la monetina"
        weight: 0.95
      - text: "butta una moneta"
        weight: 0.95
      - text: "butta la moneta"
        weight: 0.95
      - text: "butta una monetina"
        weight: 0.95
      - text: "butta la monetina"
        weight: 0.95
      - text: "fai volare una moneta"
        weight: 0.95
      - text: "fai volare la moneta"
        weight: 0.95
      - text: "fai volare una monetina"
        weight: 0.95
      - text: "fai volare la monetina"
        weight: 0.95
      - text: "lancia una moneta in aria"
        weight: 0.95
      - text: "tira una moneta in aria"
        weight: 0.95
      - text: "è testa o croce"
        weight: 0.95
      - text: "giochiamo a testa o croce"
        weight: 0.95
      - text: "dai facciamo testa o croce"
        weight: 0.95
      - text: "testa o croce dai"
        weight: 0.95
      - text: "puoi lanciare una moneta"
        weight: 0.95
      - text: "puoi tirare una moneta per me"
        weight: 0.95
      - text: "potresti lanciare una moneta"
        weight: 0.95
      - text: "potresti tirare una moneta"
        weight: 0.95
      - text: "ti va di lanciare una moneta"
        weight: 0.95
      - text: "lanci una moneta per favore"
        weight: 0.95
      - text: "lanciamela una moneta"
        weight: 0.95
      - text: "lancia una moneta per me"
        weight: 0.95
      - text: "tira una monetina per favore"
        weight: 0.95
      - text: "fai un lancio della moneta"
        weight: 0.95
      - text: "facciamo un lancio della moneta"
        weight: 0.95
      - text: "un lancio della moneta"
        weight: 0.95
      - text: "facciamo a pari o dispari con una moneta"
        weight: 0.95
      - text: "che decida la sorte"
        weight: 0.95
      - text: "che scelga il destino"
        weight: 0.95
      - text: "affidiamoci alla fortuna"
        weight: 0.95
      - text: "decidiamo con una moneta"
        weight: 0.95
      - text: "tiriamo a sorte"
        weight: 0.95
      - text: "facciamo decidere alla monetina"
        weight: 0.95
      - text: "lascia decidere alla moneta"
        weight: 0.95
      - text: "decida la moneta"
        weight: 0.95
      - text: "non so decidere lancia una moneta"
        weight: 0.95
      - text: "lancia una moneta per decidere"
        weight: 0.95
      - text: "testa o croce per decidere"
        weight: 0.95
      - text: "facciamo testa o croce per decidere"
        weight: 0.95
      - text: "decidi tu con una moneta"
        weight: 0.95
      - text: "tira una moneta e vediamo"
        weight: 0.95
      - text: "testa vince o croce vince"
        weight: 0.95
      - text: "forza lancia una moneta"
        weight: 0.95
      - text: "buttiamo una moneta per scegliere"
        weight: 0.95
      - text: "tiriamo una moneta per scegliere"
        weight: 0.95
      - text: "se esce testa decido io"
        weight: 0.95
      - text: "facciamo a monetina"
        weight: 0.95
      - text: "giochiamocela a testa o croce"
        weight: 0.95
      - text: "lancia una moneta e dimmi il risultato"
        weight: 0.85
      - text: "tira su una moneta"
        weight: 0.95
      - text: "dammi un testa o croce"
        weight: 0.95
      - text: "voglio un testa o croce"
        weight: 0.95
      - text: "risolviamola con una moneta"
        weight: 0.95
      - text: "testa oppure croce"
        weight: 0.95
      - text: "vediamo se esce testa o croce"
        weight: 0.95
      - text: "fammi vedere testa o croce"
        weight: 0.85
      - text: "lancia una moneta virtuale"
        weight: 0.95
      - text: "tira una moneta virtuale"
        weight: 0.95
      - text: "fai girare una monetina"
        weight: 0.95
      - text: "fai saltare una moneta"
        weight: 0.95
      - text: "lancia la moneta e conta"
        weight: 0.95
      - text: "testa o croce decidi tu"
        weight: 0.95
      - text: "facciamo un bel testa o croce"
        weight: 0.95
      - text: "monetina per favore"
        weight: 0.95
      - text: "una moneta al volo"
        weight: 0.95
      - text: "tira una moneta al volo"
        weight: 0.95
      - text: "lascia scegliere alla sorte"
        weight: 0.95
      - text: "che la fortuna decida"
        weight: 0.95
      - text: "lanciamo una moneta e vediamo"
        weight: 0.95
      - text: "lancia una moneta e scegli"
        weight: 0.95
      - text: "testa vinco io croce vinci tu"
        weight: 0.95
      - text: "gira la monetina per me"
        weight: 0.95
      - text: "buttiamo in aria una moneta"
        weight: 0.95
      - text: "facciamo decidere alla sorte"
        weight: 0.95
      - text: "dai un testa o croce veloce"
        weight: 0.95
      - text: "scegli tu testa o croce"
        weight: 0.95
      - text: "risolviamo con testa o croce"
        weight: 0.95
      - text: "lancia una moneta"
        weight: 0.95
      - text: "tira una moneta"
        weight: 0.95
      - text: "testa o croce"
        weight: 0.95
      - text: "lancio della moneta"
        weight: 0.95
      - text: "puoi lanciare una moneta per me"
        weight: 0.95
      - text: "tira una moneta per favore"
        weight: 0.95
      - text: "facciamo a testa o croce"
        weight: 0.95
      - text: "mi serve un testa o croce"
        weight: 0.95
      - text: "testa o croce per favore"
        weight: 0.95
      - text: "lasciamo decidere al caso"
        weight: 0.95
      - text: "lancio di una monetina"
        weight: 0.95
      - text: "fai testa o croce"
        weight: 0.95
      - text: "scegli testa o croce"
        weight: 0.95
      - text: "che decida il caso"
        weight: 0.95
      - text: "aiutami a decidere con una moneta"
        weight: 0.95
    declarative:
      response_pick: ["coinflip.heads", "coinflip.tails"]
---

# Coin Flip (Italiano)

Lancia una moneta virtuale. Restituisce "Testa." o "Croce." in modo casuale.
