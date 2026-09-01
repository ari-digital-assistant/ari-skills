---
name: navigation
description: Avvia la navigazione verso un luogo passando all'app mappe. Capisce le destinazioni ("portami a Asda", "come arrivo alla stazione", "indicazioni per il museo") e "portami a casa". Un'impostazione sceglie tra l'app mappe predefinita e la navigazione passo-passo.
license: MIT
metadata:
  ari:
    id: dev.heyari.navigation
    version: "0.1.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [navigation]
    languages: [en, it]
    specificity: high
    matching:
      patterns:
        - regex: "\\bindicazioni per\\b"
          weight: 0.95
        - regex: "\\bportami (a|al|allo|alla|ai|agli|alle|in)\\b"
          weight: 0.9
        - regex: "\\bcome (ci )?arrivo\\b"
          weight: 0.9
        - regex: "\\b(vai|andiamo) (a|al|allo|alla|ai|agli|alle|in)\\b"
          weight: 0.8
      custom_score: false
    examples:
      - text: "portami {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "accompagnami {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "guidami {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "vai {destination}"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "andiamo {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "voglio andare {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "devo andare {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "come arrivo {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "come ci arrivo {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "come si arriva {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "come faccio ad arrivare {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "quanto ci metto ad arrivare {destination}"
        weight: 0.85
        args:
          destination: "{destination}"
      - text: "qual è la strada più veloce per arrivare {destination}"
        weight: 0.85
        args:
          destination: "{destination}"
      - text: "portami {destination} in macchina"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "portami subito {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "voglio andare {destination} in auto"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "fammi arrivare {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "conducimi {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "accompagnami {destination} per favore"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "mi porti {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "puoi portarmi {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "portami di nuovo {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "come arrivo {destination} da qui"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "fammi strada {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "andiamo subito {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "naviga {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "imposta la rotta {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "voglio andare {destination} adesso"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "indicazioni per la stazione"
        weight: 0.95
        args:
          destination: "stazione"
      - text: "indicazioni per il museo"
        weight: 0.95
        args:
          destination: "museo"
      - text: "indicazioni per l'aeroporto"
        weight: 0.95
        args:
          destination: "aeroporto"
      - text: "indicazioni per l'ospedale"
        weight: 0.95
        args:
          destination: "ospedale"
      - text: "voglio tornare {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "voglio tornare {destination} in macchina"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "riportami {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "portami velocemente {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "ho bisogno di andare {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "mi serve andare {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "dobbiamo andare {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "fammi da navigatore {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "come ci si arriva {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "che strada faccio per arrivare {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "trovami la strada {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "portami {destination} adesso"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "voglio andare {destination} subito"
        weight: 0.85
        args:
          destination: "{destination}"
      - text: "accompagnami {destination} in auto"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "conducimi {destination} per favore"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "vorrei andare {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "mi indichi come arrivare {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "portaci {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "come arrivo prima {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "portami {destination} il prima possibile"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "portami a {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "portami al {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "come arrivo a {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "indicazioni per il {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "qual è la strada più veloce per la {destination}"
        weight: 0.85
        args:
          destination: "{destination}"
      - text: "accompagnami all'{destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "voglio andare al {destination}"
        weight: 0.75
        args:
          destination: "{destination}"
      - text: "guidami fino al {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "quanto ci metto ad arrivare in {destination}"
        weight: 0.85
        args:
          destination: "{destination}"
      - text: "devo raggiungere il {destination} entro un'ora"
        weight: 0.6
        args:
          destination: "{destination}"
      - text: "fammi strada fino al {destination}"
        weight: 0.95
        args:
          destination: "{destination}"
      - text: "voglio tornare a {destination} in macchina"
        weight: 0.95
        args:
          destination: "{destination}"
    settings:
      - key: navigation_mode
        label: Stile di navigazione
        type: select
        default: default_app
        help_text: "La navigazione passo-passo usa Google Maps su Android; l'app predefinita apre il luogo nell'app mappe che hai impostato."
        options:
          - value: default_app
            label: Apri nella mia app mappe predefinita
          - value: turn_by_turn
            label: Avvia la navigazione passo-passo
    wasm:
      module: skill.wasm
      memory_limit_mb: 2
---

# Navigazione

Avvia la navigazione verso una destinazione passando all'app mappe della
piattaforma. L'app mappe gestisce il percorso, la mappa e il traffico.
