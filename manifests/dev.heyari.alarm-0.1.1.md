---
name: alarm
description: Sets device alarms by handing off to your Clock app. Understands times ("set an alarm for 7am"), labels ("gym alarm at half past five") and recurrence ("wake me up at 6:30 every weekday"). Opens the Clock app for changing or listing alarms.
license: MIT
metadata:
  ari:
    id: dev.heyari.alarm
    version: "0.1.1"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [alarm]
    languages: [en, it]
    specificity: high
    matching:
      patterns:
        - regex: "\\b(set|create|add)\\b.*\\balarm\\b"
          weight: 0.95
        - regex: "\\bwake me up\\b.*\\bat\\b"
          weight: 0.9
        - regex: "\\balarm\\b.*\\b(for|at)\\b"
          weight: 0.85
        - regex: "\\b(cancel|delete|remove|turn off|stop)\\b.*\\balarm\\b"
          weight: 0.9
        - regex: "\\bwhat alarms\\b|\\blist.*\\balarm|\\balarms do i\\b"
          weight: 0.9
      custom_score: false
    examples:
      - text: "set an alarm for 7am"
      - text: "set an alarm for 6:30 every weekday"
      - text: "wake me up at half past six"
      - text: "gym alarm at 5:45"
      - text: "set an alarm for 8am on saturdays and sundays"
      - text: "cancel my 7am alarm"
      - text: "what alarms do i have"
      - text: "turn off my alarm"
      # Oblique phrasings the keyword patterns above deliberately miss —
      # these are the ones the router actually sees in production.
      - text: "i need to be up by six tomorrow"
      - text: "make sure i am awake at five thirty"
      - text: "do not let me sleep past eight"
      - text: "i have an early flight so buzz me at four am"
      - text: "get me out of bed at seven tomorrow"
      - text: "i want to be woken at quarter to seven"
      - text: "when am i being woken tomorrow"
      - text: "no need to wake me tomorrow morning"
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Alarm

Sets device alarms by handing off to the platform Clock app. The Clock app owns
scheduling, reboot persistence, snooze and ringing.

## Supported utterances

- `set an alarm for 7am` — one-shot alarm
- `set an alarm for 6:30 every weekday` — recurring
- `wake me up at half past six` — natural time phrasing
- `gym alarm at 5:45` — labelled alarm
- `cancel my alarm` / `what alarms do I have` — opens the Clock app (the
  platform API can't list or delete alarms directly)

## Notes

The `alarm` action is generic: the skill emits *what* to do; each frontend
decides *how*. On Android it maps to the `AlarmClock` intent family with
`EXTRA_SKIP_UI` so the alarm is created without leaving Ari.
