---
name: reminder
description: Sets timed reminders and untimed list items. Routes to the user's tasks app (default), calendar, or both, with optional voice-named lists like "add milk to my shopping list".
license: MIT
metadata:
  ari:
    id: dev.heyari.reminder
    version: "0.1.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [calendar, tasks]
    languages: [en]
    specificity: high
    matching:
      patterns:
        - regex: "\\bremind me\\b"
          weight: 0.95
        - regex: "\\b(set|create) (a |me )?reminder\\b"
          weight: 0.95
        - regex: "\\b(add|put) .+ (to|on) (my |the )?(shopping|grocery|todo|to-do|task|tasks|reminders?) list\\b"
          weight: 0.95
        - regex: "\\b(add|put) .+ (to|on) my \\w+ list\\b"
          weight: 0.9
        # Internal cancel round-trip: the partial-confidence card's
        # on_cancel payload emits `aricancelreminder <mode> <id>` as a
        # run_utterance. The engine routes it back here and the skill
        # calls the corresponding tasks_delete / calendar_delete host
        # capability. Weighted highest so nothing else can steal this
        # input. The `aricancelreminder` prefix is one contiguous
        # token so the engine's `normalize_input` (which strips
        # underscores/colons to spaces) leaves it unmangled.
        - regex: "^aricancelreminder\\b"
          weight: 1.0
        # Layer C clarification-card confirm round-trip: the Yes
        # button's utterance is `ariconfirmreminder <dest> <epoch_ms>
        # <title_hex>`. Carries the AI's pre-staged commit values
        # directly; skill decodes and writes the reminder without
        # another assistant round-trip. Same contiguous-alphanumeric
        # prefix trick as aricancelreminder.
        - regex: "^ariconfirmreminder\\b"
          weight: 1.0
      custom_score: false
    examples:
      - text: "remind me to walk the dog at 5pm"
      - text: "remind me to buy milk"
      - text: "remind me to take out the bins tonight"
      - text: "remind me at 9am tomorrow to call the dentist"
      - text: "remind me in 30 minutes to check the oven"
      - text: "set a reminder to email Sarah on Friday at 3"
      - text: "add milk to my shopping list"
      - text: "put eggs on the shopping list"
      - text: "add deadline review to my work list"
      - text: "remind me about the meeting at 4pm"
    settings:
      - key: destination
        label: Save reminders to
        type: select
        default: tasks
        options:
          - value: tasks
            label: Tasks
          - value: calendar
            label: Calendar
          - value: both
            label: Both
      - key: default_calendar
        label: Default calendar
        type: device_calendar
        show_when:
          key: destination
          equals: [calendar, both]
      - key: default_task_list
        label: Default task list
        type: device_task_list
        show_when:
          key: destination
          equals: [tasks, both]
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Reminder

Sets timed reminders and untimed list items, routing them to the user's
tasks app, calendar, or both based on the **Save reminders to** setting.

## Supported utterances

Default destination (uses your selected default list / calendar):

- `remind me to walk the dog at 5pm` — timed
- `remind me to buy milk` — untimed (always goes to Tasks regardless of destination)
- `remind me at 9am tomorrow to call the dentist` — relative date + explicit time
- `remind me in 30 minutes to check the oven` — relative time
- `set a reminder to email Sarah on Friday at 3` — explicit weekday

Named list (overrides the default list — voice always wins):

- `add milk to my shopping list` — named list, untimed
- `put eggs on the shopping list` — same shape, "put on" verb
- `add deadline review to my work list` — any user-named list

If no time is given the reminder is created as an untimed task. If a time
is given, it's emitted as an absolute ISO-8601 timestamp; the frontend
handles writing it as a VTODO with a due date and/or a VEVENT with an
alarm depending on the destination setting.

## Settings

- **Save reminders to** — Tasks (default), Calendar, or Both. Tasks is
  disabled if no OpenTasks-compatible app (Tasks.org, jtx Board,
  OpenTasks, etc) is installed; the settings panel shows install
  links in that case.
- **Default calendar** — picked from `CalendarContract.Calendars`.
- **Default task list** — picked from the OpenTasks ContentProvider.

## Action envelope

This skill returns `Response::Action` with the unified `v:1` envelope.
Reminder writes go through a top-level `create_reminder` slot —
matching the existing convention for side-effecting slots like
`launch_app`, `search`, `clipboard`. The `when` field is a structured
descriptor rather than an absolute timestamp — keeps the skill
timezone-naive and lets the frontend resolve against the device's
local zone:

```json
{
  "v": 1,
  "create_reminder": {
    "title": "walk the dog",
    "when": { "local_time": "17:00", "day_offset": 0 },
    "list_hint": null,
    "speak_template": "Added {title} to your {list_name} list"
  }
}
```

`when` shapes:
- `null` — untimed (always routes to Tasks regardless of the destination setting)
- `{ "in_seconds": N }` — relative ("in 30 minutes" → `1800`)
- `{ "local_time": "HH:MM", "day_offset": N }` — absolute local clock.
  `day_offset` is 0 for today, 1 for tomorrow, etc. Frontend bumps a
  bare "today at past time" to "tomorrow at that time" defensively.
- `{ "day_offset": N }` — date-only ("tomorrow" with no time) → VTODO
  with due date and no due time.

Other fields:
- `title` — the reminder text, with timing and list phrases stripped.
- `list_hint` — the spoken list name (e.g. `"shopping"`) when the user
  named one, otherwise `null`. Frontend fuzzy-matches against the user's
  available lists; on no match, falls back to the default.
- `speak_template` — spoken response with `{title}` and `{list_name}` /
  `{calendar_name}` placeholders the frontend substitutes after resolving
  the destination.

See [docs/action-responses.md](../../docs/action-responses.md) for the
shared envelope schema.

## Notes

Time parsing is English-only for v0.1. Future translations are the
skill's own responsibility — the engine and frontend stay locale-naive.

Untimed reminders always route to Tasks regardless of the
**Save reminders to** setting, since calendar grids have no useful
representation for an event without a time.
