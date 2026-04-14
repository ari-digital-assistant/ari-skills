# Action Responses

Skills can return three kinds of response: plain **Text**, a structured **Action** envelope, and **Binary** (reserved). Plain text covers almost every short-form skill ("Heads.", "Tails.", a calculation result). The Action envelope is the path for skills that need the frontend to *do* something — render a card, fire an alert, copy to the clipboard, launch an app. This doc is the contract for those envelopes.

## The envelope

One unified shape, no top-level `action` discriminator. Skills compose primitives — cards, alerts, notifications — and single-shot slots like `launch_app` or `clipboard`.

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
  "dismiss": {
    "cards": ["id", …],
    "notifications": ["id", …],
    "alerts": ["id", …]
  }
}
```

Rules:

- `"v": 1` is **required**. Frontends reject mismatched versions (catches half-migrated installs immediately).
- `speak` is optional bubble text + TTS. For `launch_app` / `search` / `open_url`, omit it and the frontend produces the platform-appropriate phrase ("Opening Spotify").
- Every primitive array is always an array (zero items = absent/empty, both fine).
- Single-shot slots (`launch_app`, `search`, `open_url`, `clipboard`) accept at most one value per envelope.
- Frontend applies `dismiss` **first**, then upserts cards/alerts/notifications by `id` (re-emitting a primitive with an existing id replaces it).
- Unknown root-level fields are ignored — additive forward-compat.

## Cards

In-chat panels with optional countdown, progress, and action buttons. The frontend picks how to render based on which fields are set; there's no "kind" enum to update when adding a variant.

```json
{
  "id": "card_t_01HZ…",
  "title": "Pasta timer",
  "subtitle": "Started just now",
  "body": null,
  "icon": "asset:timer_icon.png",
  "countdown_to_ts_ms": 1744648920000,
  "started_at_ts_ms": 1744648440000,
  "progress": null,
  "accent": "default",
  "actions": [
    {"id": "cancel", "label": "Cancel", "utterance": "cancel my pasta timer", "style": "destructive"}
  ],
  "on_complete": {
    "alert": { /* alert primitive */ },
    "dismiss_card": true,
    "dismiss_notifications": ["notif_t_…"]
  }
}
```

- `id` (required, skill-stable) — re-emit with the same id to update.
- `title` (required); `subtitle`/`body` optional.
- `icon` optional — `asset:<path>` resolves against the emitting skill's bundle.
- `countdown_to_ts_ms` (Unix ms): card renders a live countdown to that timestamp; pair with `started_at_ts_ms` for an auto-progress bar.
- `progress` `{value: 0.0..1.0}` — static progress bar (mutually exclusive with auto-progress).
- `accent` ∈ `default | warning | success | critical` — frontend maps to its color tokens.
- `actions` (0..N) — buttons; tap sends `utterance` through `engine.processInput` (see Action buttons below).
- `on_complete` (optional, only meaningful with `countdown_to_ts_ms`) — declares what happens at the deadline:
  - `alert` — fires the alert primitive.
  - `dismiss_card` (default `true`) — removes the card from the frontend's mirror.
  - `dismiss_notifications` (default `[]`) — list of notification ids to dismiss at the same instant. Use this when the skill emitted a paired ongoing notification with the card (the typical timer pattern) so the shade entry vanishes the moment the alert fires, rather than ticking past zero.

## Alerts

Loud "right now" attention-grabbing audio + notification. Frontend implementations:

- **Android**: foreground service, looping `MediaPlayer` + dedicated TTS through the alarm audio stream, `IMPORTANCE_HIGH` channel, `setFullScreenIntent` when `urgency=critical && full_takeover`.
- **Linux** (future): libnotify with `urgency=critical`, GStreamer for the audio loop.

```json
{
  "id": "alert_t_01HZ…",
  "title": "Pasta timer done",
  "body": null,
  "urgency": "critical",
  "sound": "asset:timer.mp3",
  "speech_loop": "Pasta timer",
  "auto_stop_ms": 120000,
  "max_cycles": 12,
  "full_takeover": true,
  "actions": [
    {"id": "stop_alert", "label": "Stop", "style": "primary"}
  ]
}
```

- `urgency` ∈ `normal | high | critical`.
- `sound` accepts a token (`system.alarm`, `system.notification`, `system.silent`) or an `asset:<path>` URI to a file in the emitting skill's bundle.
- `speech_loop` (optional) — TTS interjects this between sound cycles, Siri-style. Omit for sound-only loops.
- `auto_stop_ms` / `max_cycles` cap the loop. Frontend stops at whichever fires first.
- `full_takeover` requests wake-screen / full-screen-intent behavior. Frontend ignores it unless `urgency == critical` (safety gate).

## Notifications

Persistent, ambient shade entries — distinct from alerts (which grab attention now and clear themselves).

```json
{
  "id": "notif_t_01HZ…",
  "title": "Pasta timer",
  "body": "Running…",
  "importance": "default",
  "sticky": true,
  "countdown_to_ts_ms": 1744648920000,
  "actions": [
    {"id": "cancel", "label": "Cancel", "utterance": "cancel my pasta timer"}
  ]
}
```

- `importance` ∈ `min | low | default | high`. Notifications never take over the screen; that's what alerts are for.
- `sticky` → ongoing, can't be swiped away.
- `countdown_to_ts_ms` → Android renders the OS Chronometer widget with `setUsesChronometer(true) + setChronometerCountDown(true)`. Free 1Hz tick, no polling.

## Single-shot slots

For one-line side effects:

| Slot | Field | Behaviour |
|---|---|---|
| `launch_app` | `string` | Frontend resolves the app name and starts it. |
| `search` | `string` | Frontend opens the user's default web search with the query. |
| `open_url` | `string` | Frontend opens the URL via the platform browser handler. |
| `clipboard` | `{text: string}` | Frontend copies the text to the system clipboard. Doesn't replace `speak` — both can fire in one envelope. |

For `launch_app`, `search`, `open_url`, omit `speak` so the frontend can phrase the response platform-appropriately. The skill's `speak` (if set) wins as an override.

## Action buttons

Used by cards, alerts, and notifications.

```json
{"id": "cancel", "label": "Cancel", "utterance": "cancel my pasta timer", "style": "destructive"}
```

- `id` — local to the primitive. Reserved values:
  - `stop_alert` — shortcuts to AlertService stopping the active alert; frontend handles locally, no engine round-trip.
  - `dismiss_notification` — frontend clears the named notification locally.
  - Anything else routes through `engine.processInput(utterance)`.
- `label` — button text.
- `utterance` (optional, required for non-reserved ids) — the text the frontend sends to the engine when the button is tapped. The skill handles it like any other utterance and the resulting envelope flows back as a normal response.
- `style` ∈ `default | primary | destructive` — visual hint only.

## Assets

Skills can ship raw files (audio, images) under `assets/` in their bundle. Primitives reference them via `asset:<path>`.

```
skills/<your-skill>/
  SKILL.md
  skill.wasm
  assets/
    timer.mp3
    timer_icon.png
```

The frontend resolves `asset:timer_icon.png` to the file inside *your* skill's install dir — assets are namespace-scoped, one skill can't read another's. The bundle extractor enforces safe paths at install time (no `..`, no absolute paths), and frontends double-check defensively at resolution time.

Sound tokens that are not `asset:<path>` are a closed vocabulary:

| Token | Android maps to | Linux maps to (future) |
|---|---|---|
| `system.alarm` | `RingtoneManager.TYPE_ALARM` | alarm-clock-elapsed sound theme |
| `system.notification` | `RingtoneManager.TYPE_NOTIFICATION` | message-new-instant |
| `system.silent` | no audio | no audio |

Unknown tokens fall back to `system.notification` with a warning.

## Versioning

The envelope carries a `v` integer. `v: 1` is the current schema. Frontends reject mismatched versions immediately (returns "I couldn't understand that action."). Bump only on breaking schema changes; additive fields go through forward-compat (frontends ignore unknown fields).

## Emitting from Rust

```rust
use ari_skill_sdk::presentation as p;

let json = p::Envelope::new()
    .speak("Pasta timer set for 8 minutes.")
    .card(
        p::Card::new("card_t_01HZ")
            .title("Pasta timer")
            .icon(p::Asset::new("timer_icon.png"))
            .countdown_to(end_ts_ms)
            .started_at(created_ts_ms)
            .action(p::Action::new("cancel", "Cancel").utterance("cancel my pasta timer").destructive())
            .on_complete(
                p::OnComplete::new()
                    .alert(
                        p::Alert::new("alert_t_01HZ")
                            .title("Pasta timer done")
                            .urgency(p::Urgency::Critical)
                            .sound(p::Sound::asset("timer.mp3"))
                            .speech_loop("Pasta timer")
                            .full_takeover(true)
                            .action(p::Action::new("stop_alert", "Stop").primary()),
                    )
                    .dismiss_notification("notif_t_01HZ"),
            ),
    )
    .to_json();
ari::respond_action(&json)
```

The `presentation` feature is on by default in `ari-skill-sdk`. Disable it (`default-features = false`) if you're shipping a tiny text-only skill and want the leanest wasm.

## Emitting from AssemblyScript

The AS SDK doesn't yet have a typed builder for the envelope; hand-build the JSON for now and call `respondAction(json)`. Builder parity is on the roadmap.

```typescript
import { respondAction } from "ari-skill-sdk-as/assembly";

export function execute(ptr: i32, len: i32): i64 {
    const json = `{"v":1,"speak":"Opening Spotify.","launch_app":"Spotify"}`;
    return respondAction(json);
}
```

## What the Android frontend does today

| Primitive | Component |
|---|---|
| `cards[]` | `PresentationCoordinator` upserts into `CardStateRepository`; `MessageBubble` renders `GenericCard` inline; `CardAlarmScheduler` schedules the `on_complete.alert` for any card with a countdown. |
| `alerts[]` | `AlertService` (foreground service) starts immediately and runs the sound→speech loop. |
| `notifications[]` | `NotificationCoordinator` posts via `NotificationCompat.Builder`; `countdown_to_ts_ms` lights up the Chronometer widget. |
| `launch_app` | `AppLauncher` resolves to an `Intent.ACTION_MAIN`. |
| `search` | `WebSearchLauncher` opens the user's default web search. |
| `open_url` | `Intent.ACTION_VIEW` with the URI. |
| `clipboard` | System `ClipboardManager` `setPrimaryClip`. |
| `dismiss.*` | Cancel matching alarm + notification + foreground alert. |

If you need an action type Android doesn't know yet, extend `ActionHandler` or `PresentationCoordinator` — but most new use cases fit existing primitives by combining fields.

## See also

- [wasm-sdk.md](wasm-sdk.md) — full ABI contract
- [skill-system.md](skill-system.md) — end-to-end architecture
- [skill-authors.md](skill-authors.md) — skill-authoring quickstart
