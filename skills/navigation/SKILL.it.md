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
      - text: "portami a asda"
        args:
          destination: "asda"
      - text: "portami al lavoro"
        args:
          destination: "lavoro"
      - text: "portami a casa"
        args:
          destination: "casa"
      - text: "come arrivo a mcdonalds"
        args:
          destination: "mcdonalds"
      - text: "indicazioni per il museo"
        args:
          destination: "museo"
      # Frasi oblique che i pattern qui sopra non intercettano di proposito:
      # sono quelle che il router vede davvero in produzione.
      - text: "qual è la strada più veloce per la stazione"
        args:
          destination: "stazione"
      - text: "accompagnami all'ospedale"
        args:
          destination: "ospedale"
      - text: "voglio andare al mare"
        args:
          destination: "mare"
      - text: "guidami fino al museo"
        args:
          destination: "museo"
      - text: "quanto ci metto ad arrivare in aeroporto"
        args:
          destination: "aeroporto"
      - text: "devo raggiungere il centro entro un'ora"
        args:
          destination: "centro"
      - text: "fammi strada fino al ristorante"
        args:
          destination: "ristorante"
      - text: "voglio tornare a casa in macchina"
        args:
          destination: "casa"
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
