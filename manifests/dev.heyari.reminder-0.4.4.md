---
name: reminder
description: Sets timed reminders and untimed list items. Routes to the user's tasks app (default), calendar, or both, with optional voice-named lists like "add milk to my shopping list".
license: MIT
metadata:
  ari:
    id: dev.heyari.reminder
    version: "0.4.4"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [calendar, tasks]
    languages: [en, it]
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
        # Bare form, no determiner: "add milk to family shopping list".
        # Without this the keyword scorer returns 0 and the utterance
        # depends entirely on the router, which then dispatches with
        # typed args that have no list_hint in them. The trailing literal
        # "list" is what keeps this from grabbing ordinary sentences.
        #
        # Drop the word entirely — "add milk to family shopping" — and no
        # pattern here matches, so the router is the only way in. That is
        # deliberate: a regex loose enough to catch it would also catch
        # "add the lamp to the living room group". The grammar recovers
        # the list name on the args path instead, by checking the trailing
        # words against the lists the user actually has.
        - regex: "\\b(add|put) .+ (to|on) (my |the |our |your |their )?[\\w]+( [\\w]+)? list\\b"
          weight: 0.9
        # Read-only queries — list reminders for today/tomorrow, or
        # the next upcoming reminder. Patterns assume the input has
        # been through `normalize_input`, which expands `what's` →
        # `what is` and lowercases everything BEFORE the engine runs
        # the regex. So no apostrophes here, ever.
        - regex: "\\bwhat is (my|the) next reminder\\b"
          weight: 0.95
        - regex: "\\bwhat reminders? do i have\\b"
          weight: 0.9
        - regex: "\\b(do i have any|any|got any|have i got any) reminders?\\b"
          weight: 0.9
        - regex: "\\bwhat is (coming up|on my list|on today|on tomorrow)\\b"
          weight: 0.85
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
      - text: "set a reminder for me to {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "create a reminder to {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "schedule a reminder {when} for me to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "make me a reminder to {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "add a reminder to {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "put in a reminder to {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "can you set a reminder for me to {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "could you create a reminder to {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "would you set a reminder {when} to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "please schedule a reminder to {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "I need a reminder to {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "I'd like a reminder {when} to {text}"
        weight: 0.85
        args:
          title: "{text}"
          when: "{when}"
      - text: "I want a reminder to {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "set a reminder {when} so I don't forget to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "make sure I get a reminder {when} to {text}"
        weight: 0.75
        args:
          title: "{text}"
          when: "{when}"
      - text: "give me a reminder {when} to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "send me a reminder {when} to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "have a reminder pop up {when} to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "arrange a reminder {when} for me to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "leave me a reminder {when} to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "put {text} in my reminders {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "add {text} to my reminders {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "put a reminder on my schedule {when} to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "set up a reminder {when} to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "program a reminder {when} for me to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "I could use a reminder {when} to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "I need you to set a reminder {when} to {text}"
        weight: 0.85
        args:
          title: "{text}"
          when: "{when}"
      - text: "is it possible to set a reminder {when} to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "how about setting a reminder {when} to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "do me a favor and set a reminder {when} to {text}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "remind me to {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "please remind me to {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "can you remind me to {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "could you remind me to {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "don't let me forget to {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "make sure I remember to {text}"
        weight: 0.75
        args:
          title: "{text}"
      - text: "I need a reminder to {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "create a reminder for me to {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "put down a reminder to {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "jot down a reminder for me to {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "save a reminder that I need to {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "keep a reminder for me to {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "add {item} to my {list} list"
        weight: 0.75
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "put {item} on my {list} list"
        weight: 0.75
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "please add {item} to my {list} list"
        weight: 0.75
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "can you put {item} on my {list} list"
        weight: 0.75
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "I need {item} added to my {list} list"
        weight: 0.85
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "make sure {item} goes on my {list} list"
        weight: 0.75
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "stick {item} on my {list} list"
        weight: 0.95
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "pop {item} onto my {list} list"
        weight: 0.95
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "jot {item} down on my {list} list"
        weight: 0.95
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "add {item} to the {list} list"
        weight: 0.75
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "put {item} on our {list} list"
        weight: 0.75
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "add {item} to {list} list"
        weight: 0.75
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "{item} needs to go on my {list} list"
        weight: 0.85
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "remind me to {title} {when}"
        weight: 0.95
        args:
          title: "{title}"
          when: "{when}"
      - text: "remind me {when} to {title}"
        weight: 0.95
        args:
          title: "{title}"
          when: "{when}"
      - text: "set a reminder to {title} {when}"
        weight: 0.95
        args:
          title: "{title}"
          when: "{when}"
      - text: "add {title} to my {list_hint} list"
        weight: 0.75
        args:
          title: "{title}"
          list_hint: "{list_hint}"
      - text: "put {title} on the {list_hint} list"
        weight: 0.75
        args:
          title: "{title}"
          list_hint: "{list_hint}"
      - text: "add {title} to {list_hint} list"
        weight: 0.75
        args:
          title: "{title}"
          list_hint: "{list_hint}"
      - text: "remind me about {title} {when}"
        weight: 0.95
        args:
          title: "{title}"
          when: "{when}"
      - text: "ping me {when} to {title}"
        weight: 0.6
        args:
          title: "{title}"
          when: "{when}"
      - text: "tell me {when} to {title}"
        weight: 0.6
        args:
          title: "{title}"
          when: "{when}"
      - text: "buzz me about {title} {when}"
        weight: 0.95
        args:
          title: "{title}"
          when: "{when}"
      - text: "give me a shout {when} to {title}"
        weight: 0.6
        args:
          title: "{title}"
          when: "{when}"
      - text: "let me know to {title} {when}"
        weight: 0.6
        args:
          title: "{title}"
          when: "{when}"
      - text: "give me a heads up {when} to {title}"
        weight: 0.95
        args:
          title: "{title}"
          when: "{when}"
      - text: "nudge me about {title} {when}"
        weight: 0.95
        args:
          title: "{title}"
          when: "{when}"
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

Creating a reminder with no time is never a high-confidence parse (since
v0.4.0): it goes through the assistant round-trip, so you get either a
confirmation question or a card you can cancel, instead of an untimed
task filed silently. Speech recognition clipping a trailing "in one
hour" used to do exactly that. Named-list adds are unaffected — untimed
is their normal shape.

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

See [docs/reference-actions.md](../../docs/reference-actions.md) for the
shared envelope schema.

## Notes

Time parsing is locale-aware as of v0.2.0 — English ("at 5pm",
"tomorrow", "in 30 minutes") and Italian ("alle 17", "domani", "tra 30
minuti") shapes are both recognised by the same parser. Italian users
get the matching `SKILL.it.md` manifest for routing and the
`strings/it.json` table for response phrasing. Adding a third language
is a one-pass addition to the parser's union dictionaries plus a new
`SKILL.<locale>.md` and `strings/<locale>.json`.

Untimed reminders always route to Tasks regardless of the
**Save reminders to** setting, since calendar grids have no useful
representation for an event without a time.
