---
name: coin-flip
description: Flips a virtual coin and returns heads or tails. Use when the user asks to flip a coin, toss a coin, or make a random binary choice.
license: MIT
metadata:
  ari:
    id: dev.heyari.coinflip
    version: "0.1.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3,<0.4"
    capabilities: []
    languages: [en]
    specificity: high
    matching:
      patterns:
        - keywords: [flip, coin]
          weight: 0.95
        - keywords: [toss, coin]
          weight: 0.95
    examples:
      - text: "flip a coin"
      - text: "toss a coin"
      - text: "heads or tails"
      - text: "coin flip"
      - text: "can you flip a coin for me"
      - text: "toss a coin please"
      - text: "let's flip for it"
      - text: "I need a coin flip"
      - text: "heads or tails please"
      - text: "let's leave it to chance"
      - text: "coin toss"
      - text: "do a coin flip"
      - text: "pick heads or tails"
      - text: "let chance decide"
      - text: "help me decide with a coin flip"
    declarative:
      response_pick: ["Heads.", "Tails."]
---

# Coin Flip

Flips a virtual coin. Returns "Heads." or "Tails." at random.
