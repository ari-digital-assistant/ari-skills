# Action Responses

Skills can return three kinds of response: plain **Text**, structured **Action** JSON, and **Binary** (reserved). This document is the contract for Action responses — the envelope shape frontends decode, and the patterns skills use to emit them.

If you just want plain text, there is nothing here you need. `respond_text("Heads.")` is still the right answer for coin-flip-shaped skills.

## When to emit an action

Emit an action when the response needs the frontend to *do* something beyond speaking/displaying a string. Three typical cases:

- **Launch something.** `{"action":"open","target":"Spotify"}` — the Android frontend resolves the target to an installed app and starts it.
- **Trigger a rich UI.** A timer card with a live countdown, a weather card with an icon, a map preview. These need structured data the frontend renders.
- **Persist state the frontend tracks.** Reminders, timers, shopping lists — anything where the frontend mirrors the skill's state into its own UI.

If you're tempted to serialise a human-readable sentence and let the frontend regex it, emit an action instead.

## The envelope

Every action response is a JSON object with at minimum an `action` discriminator. Beyond that the shape is per-action — the frontend dispatches on `action` and each handler knows what fields to expect.

A convention that works well for skills mutating persisted state (timers, reminders, lists):

```json
{
  "action": "timer",
  "speak": "Pasta timer set for 8 minutes.",
  "events": [
    { "kind": "create", "id": "t_01HZ...", "name": "pasta",
      "duration_ms": 480000, "end_ts_ms": 1744648920000,
      "created_ts_ms": 1744648440000 }
  ],
  "timers": [ /* full authoritative snapshot */ ]
}
```

Three fields earn their keep here:

- **`speak`** — human-readable bubble text + TTS. Solves the "Text OR Action" problem at the protocol level. The frontend's action handler extracts this into the message bubble; skills don't have to duplicate it outside the envelope.
- **`events`** — what just changed. Lets the frontend apply incremental updates (schedule an alarm, dismiss a notification) without diffing snapshots itself.
- **`timers`** (or whatever your skill's noun is) — the full authoritative list *after* the mutation. Frontends reconcile against this, so a dropped event or process death mid-update self-heals on the next utterance. Cheap idempotency.

Not every action needs all three. A one-shot `"open"` action just carries `action` and `target`. Use what fits.

## Emitting an action from Rust

```rust
use ari_skill_sdk as ari;
use serde_json::json;

#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let _input = unsafe { ari::input(ptr, len) };
    let envelope = json!({
        "action": "open",
        "speak": "Opening Spotify.",
        "target": "Spotify"
    });
    ari::respond_action(&envelope.to_string())
}
```

Enable the `serde_json` dependency in your skill's `Cargo.toml` with `default-features = false, features = ["alloc"]` to keep the wasm module no_std.

## Emitting an action from AssemblyScript

```typescript
import { ari_alloc, input, respondAction } from "ari-skill-sdk-as/assembly";
export { ari_alloc };

export function execute(ptr: i32, len: i32): i64 {
    const _ = input(ptr, len);
    const envelope = '{"action":"open","speak":"Opening Spotify.","target":"Spotify"}';
    return respondAction(envelope);
}
```

AS's `JSON.stringify` works too if you prefer building an object. For small fixed payloads a template string is fine.

## Existing action handlers on Android

As of the current engine release, the Android frontend recognises:

| `action` | Fields | Behaviour |
|---|---|---|
| `open` | `target: string` | Resolve the app name against installed packages; launch it. |
| `search` | `query: string` | Open the default web search with the query. |
| `timer` | `speak`, `events`, `timers` | Mirror the timer list into `TimerStateRepository`, schedule `AlarmManager` entries, post ongoing notifications with a live `Chronometer` countdown. |

Unknown `action` values produce a generic "I don't know how to do that yet" text response — the skill still loads, it just can't drive the frontend. Add a new action type by extending `ActionHandler` on each frontend that needs to honour it.

## What the host does with the JSON

`respond_action(json)` packs the UTF-8 bytes into your skill's linear memory, prepends the `0x01` tag byte to the packed `i64` return, and the engine decodes it as `Response::Action(serde_json::Value)`. That value flows through `AriEngine::process_input` to the FFI boundary as `FfiResponse::Action { json }` and lands in the frontend's action handler exactly as you wrote it.

If the JSON doesn't parse, the engine falls back to `(skill error)` text — there's no silent mangling. Test your skill through `ari-cli --extra-skill-dir /path/to/skill "..."` before publishing; the CLI prints whatever the host decoded, so a malformed envelope is immediately visible.

## See also

- [wasm-sdk.md](wasm-sdk.md) — full ABI contract including the tag-byte packing
- [skill-system.md](skill-system.md) — end-to-end architecture, how the response reaches the frontend
- [skill-authors.md](skill-authors.md) — skill-authoring quickstart
