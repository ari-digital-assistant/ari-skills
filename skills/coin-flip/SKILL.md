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
    declarative:
      response_pick: ["Heads.", "Tails."]
---

# Coin Flip

Flips a virtual coin. Returns "Heads." or "Tails." at random.

## Example utterances

- "flip a coin"
- "toss a coin for me"
- "can you flip a coin"
