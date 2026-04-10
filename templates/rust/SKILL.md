---
name: my-skill
description: Short description of what this skill does. Second sentence explains when it should activate.
license: MIT
metadata:
  ari:
    id: com.example.myskill
    version: "0.1.0"
    author: Your Name
    engine: ">=0.3,<0.4"
    capabilities: []
    languages: [en]
    specificity: high
    matching:
      patterns:
        - keywords: [my, skill]
          weight: 0.95
      custom_score: false
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# My Skill

Describe what this skill does and how it works.

## Example utterances

- "my skill"
