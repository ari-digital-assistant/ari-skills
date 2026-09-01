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
    engine: ">=0.1"
    capabilities: []
    languages: [en, it]
    specificity: high
    matching:
      patterns:
        - keywords: [flip, coin]
          weight: 0.95
        - keywords: [toss, coin]
          weight: 0.95
    examples:
      - text: "flip a coin for me"
        weight: 0.95
      - text: "give the coin a flip"
        weight: 0.95
      - text: "flip me a coin"
        weight: 0.95
      - text: "can you toss a coin"
        weight: 0.95
      - text: "toss a coin for me"
        weight: 0.95
      - text: "throw a coin"
        weight: 0.95
      - text: "flip it"
        weight: 0.95
      - text: "toss it"
        weight: 0.95
      - text: "give it a toss"
        weight: 0.95
      - text: "do a coin toss"
        weight: 0.95
      - text: "coin flip please"
        weight: 0.95
      - text: "flip the coin"
        weight: 0.95
      - text: "toss the coin"
        weight: 0.95
      - text: "lets flip a coin"
        weight: 0.95
      - text: "lets toss a coin"
        weight: 0.95
      - text: "flip a coin quick"
        weight: 0.95
      - text: "quick coin flip"
        weight: 0.95
      - text: "just flip a coin"
        weight: 0.95
      - text: "flip a coin and tell me"
        weight: 0.95
      - text: "heads or tails then"
        weight: 0.95
      - text: "call it heads or tails"
        weight: 0.95
      - text: "is it heads or tails"
        weight: 0.95
      - text: "heads or tails go on"
        weight: 0.95
      - text: "lets do heads or tails"
        weight: 0.95
      - text: "give me heads or tails"
        weight: 0.95
      - text: "whats it going to be heads or tails"
        weight: 0.95
      - text: "flip for heads or tails"
        weight: 0.95
      - text: "call heads or tails for me"
        weight: 0.95
      - text: "lets settle this with a coin"
        weight: 0.95
      - text: "settle it with a coin flip"
        weight: 0.95
      - text: "flip a coin to decide"
        weight: 0.95
      - text: "toss a coin to decide"
        weight: 0.95
      - text: "decide this with a coin flip"
        weight: 0.95
      - text: "flip a coin and let fate decide"
        weight: 0.95
      - text: "let the coin decide"
        weight: 0.95
      - text: "leave it to the coin"
        weight: 0.95
      - text: "let a coin flip decide"
        weight: 0.95
      - text: "coin flip to settle it"
        weight: 0.95
      - text: "flip a coin to settle the bet"
        weight: 0.95
      - text: "toss up"
        weight: 0.95
      - text: "give me a toss up"
        weight: 0.95
      - text: "do a toss up"
        weight: 0.95
      - text: "lets have a toss up"
        weight: 0.95
      - text: "call it in the air"
        weight: 0.95
      - text: "flip a coin call it"
        weight: 0.95
      - text: "flip and ill call it"
        weight: 0.95
      - text: "lets flip for who goes first"
        weight: 0.95
      - text: "flip a coin to see who wins"
        weight: 0.95
      - text: "toss a coin to break the tie"
        weight: 0.95
      - text: "break the tie with a coin flip"
        weight: 0.95
      - text: "flip a coin i cant decide"
        weight: 0.95
      - text: "i cant decide flip a coin"
        weight: 0.95
      - text: "help me pick flip a coin"
        weight: 0.95
      - text: "flip a coin its too close to call"
        weight: 0.95
      - text: "cant make my mind up flip a coin"
        weight: 0.85
      - text: "flip a coin best of three"
        weight: 0.95
      - text: "do the coin flip thing"
        weight: 0.95
      - text: "give the coin a toss"
        weight: 0.95
      - text: "spin a coin"
        weight: 0.95
      - text: "flick a coin"
        weight: 0.95
      - text: "flick a coin for me"
        weight: 0.95
      - text: "toss a coin real quick"
        weight: 0.95
      - text: "flip a coin real quick"
        weight: 0.95
      - text: "one coin flip please"
        weight: 0.95
      - text: "another coin flip"
        weight: 0.95
      - text: "flip again"
        weight: 0.95
      - text: "toss again"
        weight: 0.95
      - text: "flip the coin one more time"
        weight: 0.95
      - text: "lets flip once more"
        weight: 0.95
      - text: "go on flip a coin"
        weight: 0.95
      - text: "would you flip a coin"
        weight: 0.95
      - text: "could you toss a coin for me"
        weight: 0.95
      - text: "mind flipping a coin"
        weight: 0.95
      - text: "fancy flipping a coin"
        weight: 0.95
      - text: "flip a coin will you"
        weight: 0.95
      - text: "chuck a coin"
        weight: 0.95
      - text: "flip a coin and see"
        weight: 0.95
      - text: "lets see heads or tails"
        weight: 0.95
      - text: "gimme a coin flip"
        weight: 0.95
      - text: "gimme heads or tails"
        weight: 0.95
      - text: "call the coin"
        weight: 0.95
      - text: "flip the coin for me please"
        weight: 0.95
      - text: "toss the coin for me please"
        weight: 0.95
      - text: "lets flip on it"
        weight: 0.95
      - text: "flip on it"
        weight: 0.95
      - text: "flip a coin ill go with that"
        weight: 0.95
      - text: "random heads or tails"
        weight: 0.95
      - text: "pick one heads or tails"
        weight: 0.95
      - text: "flip a coin to choose"
        weight: 0.95
      - text: "toss a coin to choose"
        weight: 0.95
      - text: "flip a coin"
        weight: 0.95
      - text: "toss a coin"
        weight: 0.95
      - text: "heads or tails"
        weight: 0.95
      - text: "coin flip"
        weight: 0.95
      - text: "can you flip a coin for me"
        weight: 0.95
      - text: "toss a coin please"
        weight: 0.95
      - text: "let's flip for it"
        weight: 0.95
      - text: "I need a coin flip"
        weight: 0.95
      - text: "heads or tails please"
        weight: 0.95
      - text: "let's leave it to chance"
        weight: 0.85
      - text: "coin toss"
        weight: 0.95
      - text: "do a coin flip"
        weight: 0.95
      - text: "pick heads or tails"
        weight: 0.95
      - text: "let chance decide"
        weight: 0.95
      - text: "help me decide with a coin flip"
        weight: 0.95
    declarative:
      response_pick: ["coinflip.heads", "coinflip.tails"]
---

# Coin Flip

Flips a virtual coin. Returns "Heads." or "Tails." at random.
