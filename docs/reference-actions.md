# Action envelope reference

Most skills answer with a sentence. When you need more — a card, a ringing
alert, a notification, an app launch, a follow-up question — you return an
**action envelope** instead.

The deal: you describe *what you want*, in platform-neutral primitives. Each
frontend renders it its own way. No Android code, no GTK code, ever lives in
a skill.

```rust
use ari_skill_sdk::presentation as p;

let json = p::Envelope::new()
    .speak("Copied that for you.")
    .clipboard("the text")
    .to_json();
ari::respond_action(&json)
```

Declarative skills put a static envelope under `declarative.action`.

---

## The envelope

```json
{
  "v": 1,
  "speak": "Pasta timer set for 8 minutes.",
  "cards": [ … ],
  "alerts": [ … ],
  "notifications": [ … ],
  "launch_app": "Spotify",
  "search": "capital of malta",
  "open_url": "https://…",
  "clipboard": { "text": "…" },
  "alarm": { … },
  "navigate": { … },
  "media": { … },
  "await_reply": { "context": "…" },
  "run_utterance": "…",
  "confidence": "partial",
  "unparsed": "…",
  "dismiss": { "cards": [], "notifications": [], "alerts": [] }
}
```

Rules:

- **`"v": 1` is required.** A missing or mismatched version is rejected
  outright and the user gets "I couldn't understand that action."
- `speak` is the bubble text and what gets spoken. For `launch_app`, `search`
  and `open_url`, omit it and the frontend phrases it itself ("Opening
  Spotify"). Set it to override.
- Single-value slots (`launch_app`, `search`, `open_url`, `clipboard`,
  `alarm`, `navigate`, `media`) take at most one value per envelope.
- `dismiss` is applied **first**, then primitives are upserted by `id`.
  Re-emitting an id replaces what's on screen.
- Unknown fields are ignored, so new ones can be added without breaking old
  skills.

Several slots are capability-gated — see
[reference-capabilities.md](reference-capabilities.md).

## Cards

In-chat panels. The frontend picks a layout from which fields you set; there's
no "kind" enum.

```json
{
  "id": "card_pasta",
  "title": "Pasta timer",
  "subtitle": "Started just now",
  "body": null,
  "icon": "asset:timer_icon.png",
  "countdown_to_ts_ms": 1744648920000,
  "started_at_ts_ms": 1744648440000,
  "progress": { "value": 0.4 },
  "accent": "default",
  "actions": [ … ],
  "on_complete": { … },
  "on_cancel": { … },
  "stat": { … },
  "list": { … }
}
```

| Field | Notes |
|---|---|
| `id` | **Required.** Stable and skill-chosen. Re-emit to update in place. |
| `title` | **Required.** |
| `subtitle`, `body` | Optional text. |
| `icon` | `asset:<path>` into your own bundle. |
| `countdown_to_ts_ms` | Unix ms. Renders a live countdown. |
| `started_at_ts_ms` | Pair with the above for an automatic progress bar. |
| `progress` | `{value: 0.0–1.0}`. A static bar; mutually exclusive with auto-progress. |
| `accent` | `default` \| `warning` \| `success` \| `critical`. |
| `actions` | Buttons. See [action buttons](#action-buttons). |
| `on_complete` | What happens when the countdown hits zero. |
| `on_cancel` | A whole envelope, re-dispatched when the user taps Cancel. |
| `stat` | The [stat layout](#stat-cards). |
| `list` | The [list layout](#list-cards). |

**Use a stable `id`.** A random one per utterance stacks a new card up the
screen every time instead of updating the one that's there.

### `on_complete`

```json
"on_complete": {
  "alert": { … },
  "dismiss_card": true,
  "dismiss_notifications": ["notif_pasta"]
}
```

- `alert` — fires this alert primitive at the deadline.
- `dismiss_card` — defaults to `true`. **With an `alert` set**, the frontend
  keeps the card visible while the alert rings, so the Stop button is right
  where the user is looking, and removes it when the alert ends.
- `dismiss_notifications` — ids to clear at the same instant. Use it when you
  emitted a paired ongoing notification, so the shade entry vanishes rather
  than ticking past zero.

While a card's alert is ringing the frontend automatically swaps that card's
buttons for a single **Stop**. You don't need to do anything.

### Stat cards

A big number with supporting detail.

```json
"stat": {
  "headline": "7",
  "caption": "cups of water",
  "pill": { "icon": "asset:goal.png", "text": "7 of 8" },
  "metrics": [ { "text": "Best: 11" }, { "text": "Streak: 4 days" } ],
  "background": "asset:water_bg.webp",
  "footer": { "text": "Updated just now" }
}
```

`headline` is required. `pill`, `footer` and each `metrics` entry are
`{icon?, text}` pairs.

```rust
p::Card::new("tally")
    .title("Tally")
    .stat(
        p::Stat::new("7")
            .caption("cups of water")
            .pill(p::IconText::new("7 of 8"))
            .metric(p::IconText::new("Best: 11")),
    )
```

### List cards

Rows with optional leading/trailing text and badges.

```json
"list": {
  "summary": { "text": "3 reminders today" },
  "rows": [
    { "leading": "09:00", "text": "Call the dentist", "trailing": "Work",
      "badge": { "text": "Due" } }
  ],
  "footer": { "text": "Tap to open" }
}
```

`leading` is the only required field on a row.

```rust
p::Card::new("reminders")
    .title("Today")
    .list(
        p::ListCard::new()
            .summary(p::IconText::new("3 reminders today"))
            .row(p::ListRow::new("09:00").text("Call the dentist").trailing("Work")),
    )
```

## Alerts

Loud, attention-now audio plus a notification. Distinct from a notification,
which is ambient and patient.

```json
{
  "id": "alert_pasta",
  "title": "Pasta timer done",
  "body": null,
  "urgency": "critical",
  "sound": "asset:timer.mp3",
  "speech_loop": "Pasta timer",
  "auto_stop_ms": 120000,
  "max_cycles": 12,
  "full_takeover": true,
  "icon": "asset:timer_icon.png",
  "actions": [ { "id": "stop_alert", "label": "Stop", "style": "primary" } ]
}
```

| Field | Notes |
|---|---|
| `id`, `title` | **Required.** |
| `urgency` | `normal` \| `high` \| `critical`. |
| `sound` | `asset:<path>`, or a token: `system.alarm`, `system.notification`, `system.silent`. Unknown tokens fall back to `system.notification`. |
| `speech_loop` | TTS spoken between sound cycles, Siri-style. Omit for sound only. |
| `auto_stop_ms`, `max_cycles` | Caps. Whichever fires first wins. **Set at least one.** |
| `full_takeover` | Requests lock-screen takeover. **Ignored unless `urgency` is `critical`.** Needs the `critical_alert` capability. |
| `icon` | Rendered large on the takeover surface. |

A `full_takeover` critical alert gets a dedicated alarm-clock surface: live
clock, your icon with a pulsing ring, title and body, urgency-tinted
background, and your actions as full-width buttons — over the lock screen,
without unlocking.

This is intrusive. Use it for things the user has explicitly asked to be
interrupted for, and nothing else.

## Notifications

Persistent shade entries.

```json
{
  "id": "notif_pasta",
  "title": "Pasta timer",
  "body": "Running…",
  "importance": "default",
  "sticky": true,
  "countdown_to_ts_ms": 1744648920000,
  "actions": [ … ]
}
```

- `importance` — `min` \| `low` \| `default` \| `high`. Notifications never
  take over the screen; that's what alerts are for.
- `sticky` — ongoing, can't be swiped away.
- `countdown_to_ts_ms` — on Android this drives the OS chronometer widget: a
  free 1 Hz tick with no polling from you.

## Single-value slots

| Slot | Shape | Capability | Behaviour |
|---|---|---|---|
| `launch_app` | `string` | `launch_app` | Resolves an app by name and starts it. |
| `search` | `string` | — | Opens the user's default web search. |
| `open_url` | `string` | — | Opens the URL in the platform browser. |
| `clipboard` | `{text}` | `clipboard` | Copies to the system clipboard. |
| `alarm` | see below | `alarm` | Sets or shows a device alarm. |
| `navigate` | see below | `navigation` | Starts navigation. |
| `media` | see below | `media_control` | Plays or controls music. |

### `alarm`

```json
{ "op": "set", "hour": 7, "minute": 0, "message": "Wake up",
  "days": ["mon", "fri"], "skip_ui": true }
```

- `op` — `"set"` creates one; `"show"` opens the alarm list (used for cancel
  and list, which the platform API can't do directly).
- `days` — lowercase 3-letter codes `mon`–`sun`. Omitted means one-shot.
- `skip_ui` — create without showing the Clock UI. Defaults to `true`. Some
  Clock apps ignore it.

### `navigate`

```json
{ "destination": "the airport", "mode": "turn_by_turn" }
```

`mode` is `"default_app"` (the default) or `"turn_by_turn"`. `destination` is
free text, already lowercased by normalisation; the frontend URL-encodes it.

### `media`

```json
{ "action": "play", "query": "brian eno", "service": "spotify" }
```

Fields: `action` (required), `query`, `service`, `direction`, `level`, `mute`
— the last three for transport and volume control.

> The Rust SDK has **no typed builder for `media`** yet. Build the JSON by
> hand, as [`skills/music`](../skills/music) does.

## `await_reply` — asking a follow-up

A skill can ask a question and get the user's next utterance routed straight
back to itself, instead of going through matching again.

```rust
p::Envelope::new()
    .speak("Which service — Spotify or YouTube Music?")
    .await_reply(r#"{"query":"brian eno"}"#)   // opaque; you choose the shape
    .to_json()
```

The next thing the user says arrives at your `execute` wrapped up:

```json
{"_ari_reply": {"context": "{\"query\":\"brian eno\"}", "text": "spotify"}}
```

Handle it by calling `parse_reply` first and falling through if it's `None`:

```rust
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };

    if let Some(reply) = ari::parse_reply(input) {
        // reply.context is exactly the string you passed to await_reply
        // reply.text is what the user just said
        return handle_answer(&reply);
    }

    // …normal handling…
}
```

`context` is an opaque blob you round-trip through the user's answer — put
whatever state you need in it. This is the whole multi-turn mechanism; there
is no session object.

On a voice frontend the microphone re-arms automatically after the question,
so the user can just answer. Make sure you `speak` the question — a silent
`await_reply` leaves the user staring at a live mic with no idea why.

## Action buttons

Used by cards, alerts and notifications.

```json
{ "id": "cancel", "label": "Cancel", "utterance": "cancel my pasta timer",
  "speak": "Cancelling.", "style": "destructive" }
```

| Field | Notes |
|---|---|
| `id` | **Required.** Two values are reserved — see below. |
| `label` | **Required.** Button text. |
| `utterance` | Required for non-reserved ids. Sent back through the engine when tapped, exactly as if the user had said it. |
| `speak` | Optional immediate feedback on tap. |
| `style` | `default` \| `primary` \| `destructive`. Visual hint only. |

Reserved ids, handled entirely by the frontend with no engine round-trip:

- `stop_alert` — silences the active alert.
- `dismiss_notification` — clears the notification.

Everything else round-trips through `processInput(utterance)`, so your skill
handles a tap the same way it handles speech. One code path.

## `run_utterance`

An utterance the frontend re-dispatches through the engine after applying the
rest of the envelope. Handy inside `on_cancel` so a skill can round-trip a
cancel back to itself without a bespoke primitive.

## `confidence` and `unparsed`

Tell the frontend how sure you are of your own parse.

```json
{ "v": 1, "speak": "Timer set for 8 minutes.",
  "confidence": "partial", "unparsed": "and text sam when it's done" }
```

- `confidence` — `high` (the default when absent) \| `partial` \| `low`.
- `unparsed` — the bit you noticed but couldn't act on. Null when `high`.

Use `partial` when you did something useful but dropped part of the request.
The frontend surfaces the leftover rather than letting the user believe you
handled all of it.

## Assets

Ship files under `assets/` in your skill directory and reference them with
`asset:<path>`.

```
skills/your-skill/
  SKILL.en.md
  skill.wasm
  assets/
    timer.mp3
    timer_icon.png
```

Assets resolve inside *your* skill's install directory — one skill can't read
another's. The extractor rejects unsafe paths (`..`, absolute) at install
time.

Keep them small. **The whole bundle is capped at 8 MiB**, and going over it
fails the install with a misleading "couldn't reach the registry" message.

| Sound token | Android | Linux (planned) |
|---|---|---|
| `system.alarm` | `RingtoneManager.TYPE_ALARM` | alarm-clock-elapsed |
| `system.notification` | `RingtoneManager.TYPE_NOTIFICATION` | message-new-instant |
| `system.silent` | no audio | no audio |

## Versioning

`v: 1` is current. Frontends reject anything else immediately — which catches
half-migrated installs at once instead of letting them misbehave quietly.
Additive fields don't bump it; unknown fields are ignored.

## From AssemblyScript

No typed builder. Hand-build the JSON:

```typescript
import { respondAction } from "ari-skill-sdk-as/assembly";

export function execute(ptr: i32, len: i32): i64 {
  return respondAction(`{"v":1,"speak":"Opening Spotify.","launch_app":"Spotify"}`);
}
```

## See also

- [reference-capabilities.md](reference-capabilities.md) — which slots need which capability
- [reference-sdk.md](reference-sdk.md) — the `presentation` builder API
- [`templates/countdown`](../templates/countdown) — a working card + alert skill
