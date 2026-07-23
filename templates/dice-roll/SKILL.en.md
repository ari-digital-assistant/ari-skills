---
name: dice-roll
description: Rolls a six-sided dice and returns the number. Use when the user asks to roll a dice, roll a die, throw the dice, or wants a random number between one and six.
license: MIT
metadata:
  ari:
    id: com.example.diceroll
    version: "0.1.0"
    author: Your Name <you@example.com>
    homepage: https://example.com/dice-roll
    engine: ">=0.3,<0.4"
    capabilities: []
    languages: [en]
    specificity: high
    matching:
      patterns:
        - keywords: [roll, dice]
          weight: 0.95
        - keywords: [roll, die]
          weight: 0.95
        - keywords: [throw, dice]
          weight: 0.95
    examples:
      - text: "give me a number between one and six"
      - text: "i need a random number for the board game"
      - text: "pick a number, one to six"
      - text: "decide for me, one through six"
      - text: "what should i move on the board"
      - text: "settle this with a dice"
      - text: "i cannot find the dice, do it for me"
      - text: "random number please, six sided"
    declarative:
      response_pick:
        - dice.one
        - dice.two
        - dice.three
        - dice.four
        - dice.five
        - dice.six
---

# Dice Roll

Rolls a six-sided dice. "roll a dice" → "You rolled a four."

`response_pick` chooses one entry at random per invocation. Each entry here is
a key into `strings/en.json` rather than literal text, so the same manifest
serves every locale — see [i18n](../../docs/i18n.md).

## Using this template

1. Copy the directory and rename it. The directory name and the `name:` field
   must match.
2. Change `metadata.ari.id` to a reverse-DNS id under a domain you control.
3. Rewrite `description` — the first sentence says what the skill does, the
   second says when to use it. The router reads this.
4. Replace the patterns, the examples, and the `strings/en.json` entries.
5. Run `./tools/validate templates/dice-roll` (with your path) before opening a PR.

Full walkthrough: [tutorial-declarative.md](../../docs/tutorial-declarative.md).
