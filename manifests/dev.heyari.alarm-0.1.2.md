---
name: alarm
description: Sets device alarms by handing off to your Clock app. Understands times ("set an alarm for 7am"), labels ("gym alarm at half past five") and recurrence ("wake me up at 6:30 every weekday"). Opens the Clock app for changing or listing alarms.
license: MIT
metadata:
  ari:
    id: dev.heyari.alarm
    version: "0.1.2"
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
      - text: "i have an early start so wake me at {time}"
        weight: 0.85
      - text: "set an alarm for {time}"
        weight: 0.95
      - text: "set an alarm for {time} every weekday"
        weight: 0.95
      - text: "set an alarm for {time} on saturday"
        weight: 0.95
      - text: "set an alarm for {time} on weekends"
        weight: 0.95
      - text: "put an alarm on for {time}"
        weight: 0.95
      - text: "add an alarm at {time}"
        weight: 0.95
      - text: "create an alarm for {time}"
        weight: 0.95
      - text: "set a {label} alarm for {time}"
        weight: 0.95
      - text: "{label} alarm at {time}"
        weight: 0.95
      - text: "wake me at {time}"
        weight: 0.95
      - text: "wake me up at {time}"
        weight: 0.95
      - text: "wake me up at {time} tomorrow"
        weight: 0.95
      - text: "can you wake me at {time}"
        weight: 0.95
      - text: "i need to be up by {time}"
        weight: 0.6
      - text: "get me up at {time}"
        weight: 0.6
      - text: "buzz me at {time}"
        weight: 0.95
      - text: "give me a wake up call at {time}"
        weight: 0.75
      - text: "dont let me sleep past {time}"
        weight: 0.95
      - text: "make sure im awake by {time}"
        weight: 0.6
      - text: "rouse me at {time}"
        weight: 0.95
      - text: "i have to be up at {time} for my {label}"
        weight: 0.55
      - text: "set an alarm so im up for {label} at {time}"
        weight: 0.95
      - text: "i want to wake up at {time}"
        weight: 0.75
      - text: "alarm for {time} please"
        weight: 0.95
      - text: "need an alarm at {time}"
        weight: 0.95
      - text: "set the alarm for {time}"
        weight: 0.95
      - text: "wake me before {time}"
        weight: 0.95
      - text: "set an alarm for {time} and another one later"
        weight: 0.85
      - text: "schedule an alarm for {time}"
        weight: 0.95
      - text: "set a daily alarm for {time}"
        weight: 0.95
      - text: "wake me up every day at {time}"
        weight: 0.85
      - text: "set a recurring alarm for {time}"
        weight: 0.95
      - text: "get me out of bed at {time}"
        weight: 0.75
      - text: "drag me out of bed at {time}"
        weight: 0.95
      - text: "i cant miss my {label} so wake me at {time}"
        weight: 0.85
      - text: "set an alarm for {time} on monday"
        weight: 0.95
      - text: "alarm at {time} for the {label}"
        weight: 0.95
      - text: "wake me up at {time} on the dot"
        weight: 0.95
      - text: "i need waking at {time}"
        weight: 0.95
      - text: "set my morning alarm for {time}"
        weight: 0.85
      - text: "can you set an alarm for {time}"
        weight: 0.95
      - text: "cancel my alarm"
        weight: 0.95
      - text: "cancel my {time} alarm"
        weight: 0.95
      - text: "turn off my alarm"
        weight: 0.95
      - text: "turn off the {time} alarm"
        weight: 0.95
      - text: "delete my {label} alarm"
        weight: 0.95
      - text: "remove the alarm for {time}"
        weight: 0.95
      - text: "stop my morning alarm"
        weight: 0.95
      - text: "what alarms do i have"
        weight: 0.95
      - text: "what alarms do i have set"
        weight: 0.95
      - text: "do i have any alarms set"
        weight: 0.95
      - text: "when am i being woken tomorrow"
        weight: 0.95
      - text: "list my alarms"
        weight: 0.95
      - text: "have i got an alarm for the morning"
        weight: 0.85
      - text: "set an alarm for 7am"
        weight: 0.85
      - text: "set an alarm for 6:30 every weekday"
        weight: 0.95
      - text: "wake me up at half past six"
        weight: 0.95
      - text: "gym alarm at 5:45"
        weight: 0.85
      - text: "set an alarm for 8am on saturdays and sundays"
        weight: 0.75
      - text: "cancel my 7am alarm"
        weight: 0.95
      - text: "i need to be up by six tomorrow"
        weight: 0.85
      - text: "make sure i am awake at five thirty"
        weight: 0.85
      - text: "do not let me sleep past eight"
        weight: 0.95
      - text: "i have an early flight so buzz me at four am"
        weight: 0.85
      - text: "get me out of bed at seven tomorrow"
        weight: 0.95
      - text: "i want to be woken at quarter to seven"
        weight: 0.95
      - text: "no need to wake me tomorrow morning"
        weight: 0.75
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
