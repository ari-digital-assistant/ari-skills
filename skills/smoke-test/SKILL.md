---
name: smoke-test
description: Throwaway skill used to smoke-test the auto-merge pipeline. Safe to delete. Use when the user says the nonsense phrase arismoketest.
license: MIT
metadata:
  ari:
    id: dev.heyari.smoketest
    version: "0.0.1"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.1"
    capabilities: []
    languages: [en]
    specificity: high
    matching:
      patterns:
        - keywords: [arismoketest]
          weight: 0.99
    examples:
      - text: "arismoketest"
      - text: "run arismoketest"
      - text: "do arismoketest"
      - text: "arismoketest please"
      - text: "trigger arismoketest"
    declarative:
      response_pick: ["Smoke test OK."]
---

# Smoke Test

Throwaway skill used to verify the label-driven auto-merge pipeline. Returns a
fixed response. Safe to delete once the pipeline is confirmed working.
