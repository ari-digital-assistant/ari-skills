# Capability reference

A capability is permission to do something outside your sandbox. You declare
what you need; the frontend declares what it provides; the engine installs
your skill only if your set is a subset of theirs.

```yaml
metadata:
  ari:
    capabilities: [http, storage_kv]
```

Declare exactly what you use. An undeclared capability fails at install; an
unused declaration will be questioned in review.

## The fifteen

| Capability | Kind | SDK feature | Android |
|---|---|---|---|
| [`http`](#http) | Host import | `http` | ✅ |
| [`storage_kv`](#storage_kv) | Host import | `storage` | ✅ |
| [`location`](#location) | Host import | `location` | ✅ |
| [`authorize`](#authorize) | Host import | `authorize` | ✅ |
| [`media_services`](#media_services) | Host import | `media` | ✅ |
| [`tasks`](#tasks) | Host import | `tasks` | ✅ |
| [`calendar`](#calendar) | Host import | `calendar` | ✅ |
| [`notifications`](#notifications) | Frontend | — | ✅ |
| [`launch_app`](#launch_app) | Frontend | — | ✅ |
| [`clipboard`](#clipboard) | Frontend | — | ✅ |
| [`tts`](#tts) | Frontend | — | ✅ |
| [`critical_alert`](#critical_alert) | Frontend | — | ✅ |
| [`alarm`](#alarm) | Frontend | — | ✅ |
| [`navigation`](#navigation) | Frontend | — | ✅ |
| [`media_control`](#media_control) | Frontend | — | ✅ |

**Kind** is the thing to understand:

- **Host import** — your WASM module calls a function the host provides. You
  need the capability *and* the matching SDK feature flag. Declarative skills
  can't use these.
- **Frontend** — no import, no code. The capability is permission to put a
  particular slot in an [action envelope](reference-actions.md). Declarative
  skills can use these perfectly well.

Android grants all fifteen. The CLI grants `notifications`, `launch_app`,
`clipboard` and `tts` by default — pass `--host-capabilities` for anything
else. Linux grants nothing yet, because there's no Linux frontend yet.

## What you get for free

These need **no** capability. Every skill can call them:

`log` · `get_capability` · `now_ms` · `rand_u64` · `local_now_components` ·
`local_timezone_id` · `setting_get` · `setting_set` · `args` · `get_locale` ·
`t` · `format_date` · `format_number` · `format_currency` ·
`oauth_redirect_uri`

Note `setting_get`/`setting_set` in that list. Reading and writing your own
skill's settings is ungated — it's only the separate `storage_kv` scratch
store that needs permission.

---

## Host-import capabilities

### `http`

Outbound HTTP from inside the sandbox.

- **Imports:** `http_fetch`, `http_request`
- **SDK:** `features = ["http"]`

```rust
pub struct HttpResponse {
    pub status: u16,             // 0 on transport failure, not an HTTP status
    pub body: Option<String>,
    pub error: Option<String>,   // set only on transport failure
}

let resp = ari::http_fetch("https://api.example.com/data");

let resp = ari::http_request(
    "POST",
    "https://api.example.com/thing",
    &[("Authorization", "Bearer …"), ("Content-Type", "application/json")],
    Some(r#"{"hello":"world"}"#),
);
```

Two things to get right:

- **`status == 0` means the request never left.** It is not an HTTP status.
  Users on flaky mobile data will hit this constantly — handle it separately
  from a 4xx/5xx.
- **The host enforces HTTPS by default** and caps response body size. A plain
  `http://` URL will not go through.

### `storage_kv`

A private string→string store, scoped to your skill id. Survives across
invocations; wiped when the user uninstalls the skill.

- **Imports:** `storage_get`, `storage_set`
- **SDK:** `features = ["storage"]`

```rust
let count = ari::storage_get("count").unwrap_or("0");
let ok: bool = ari::storage_set("count", "1");
```

No other skill can see your namespace. `storage_set` returns `false` on
failure — check it.

### `location`

Coarse device location. Fine location is never requested.

- **Import:** `location_current`
- **SDK:** `features = ["location"]`

```rust
let loc = ari::location();
match loc.status {
    ari::LocationStatus::Ok => { /* loc.lat, loc.lon, loc.accuracy_m, loc.timestamp_ms */ }
    ari::LocationStatus::PermissionDenied => { /* tell the user why you need it */ }
    ari::LocationStatus::Unavailable | ari::LocationStatus::Timeout => { /* fall back */ }
}
```

`ari::location_with(max_age_ms, timeout_ms)` overrides the defaults (a 10
minute cached-fix window, a 5 second active-fix timeout). Prefer a generous
`max_age_ms` — a cached fix is instant and free, a fresh one costs the user
battery.

On hosts with no location provider the status is always `Unavailable`. Handle
it; don't assume.

### `authorize`

Opens an authorization URL in the system browser, waits for the redirect, and
hands you the callback query parameters. Used for OAuth 2.0 and IndieAuth
sign-in.

- **Import:** `authorize`
- **SDK:** `features = ["authorize"]` (plus `crypto` for PKCE)

**Your skill owns the protocol** — building the URL, the `state` check, PKCE,
swapping the code for a token. The host only drives the browser.

```rust
let redirect = ari::oauth_redirect_uri();   // never hardcode this
let res = ari::authorize(&auth_url, &redirect, 300_000);
if res.ok {
    let code = res.get("code").unwrap_or("");
    // verify state, then exchange `code` via ari::http_request
}
```

`res.error` is one of `"cancelled"`, `"timeout"`, `"no_browser"`,
`"mismatch"`, `"bad_request"`, `"bad_response"`. Map them to friendly text.

You don't need to own a domain or host anything: call
`ari::oauth_redirect_uri()` and register *that* with your provider. Full
worked example: [reference-manifest.md](reference-manifest.md#oauth-sign-in).

### `media_services`

Returns the canonical ids of music services installed on the device, so you
can pick a sensible target instead of guessing.

- **Import:** `media_services`
- **SDK:** `features = ["media"]`

```rust
let installed: Vec<String> = ari::media_services();
```

Distinct from [`media_control`](#media_control), which is permission to *emit*
a playback action. A skill that asks "which app?" needs both.

### `tasks`

Read and write the platform task provider — OpenTasks on Android, surfaced by
Tasks.org, jtx Board and friends.

- **Imports:** `tasks_provider_installed`, `tasks_list_lists`, `tasks_insert`,
  `tasks_delete`, `tasks_query_in_range`
- **SDK:** `features = ["tasks"]`

Always call `tasks_provider_installed` first. Plenty of devices have no task
provider at all, and the honest response is to say so rather than fail.

Pair with a `device_task_list` settings field to let the user choose which
list you write to — the frontend populates it.

### `calendar`

Read and write the platform calendar (`CalendarContract` on Android).

- **Imports:** `calendar_has_write_permission`, `calendar_list_calendars`,
  `calendar_insert`, `calendar_delete`, `calendar_query_in_range`
- **SDK:** `features = ["calendar"]`

Call `calendar_has_write_permission` before attempting a write. The runtime
permission is the user's to grant and they may not have.

Pair with a `device_calendar` settings field for calendar choice.

---

## Frontend capabilities

No imports, no SDK feature, no code. These gate [action
envelope](reference-actions.md) slots, and declarative skills can use them via
`declarative.action`.

### `notifications`

Post a notification to the shade via the `notifications[]` array. Persistent
and ambient — for anything that must grab attention *now*, use an alert.

### `launch_app`

Start another app by name.

```json
{ "v": 1, "launch_app": "Spotify" }
```

Omit `speak` and the frontend phrases it appropriately for the platform.

### `clipboard`

```json
{ "v": 1, "speak": "Copied.", "clipboard": { "text": "…" } }
```

### `tts`

Trigger speech playback. Note you do **not** need this for an ordinary spoken
response — `speak` and plain text responses are always spoken. This covers
skills that drive TTS as an effect in its own right.

### `critical_alert`

Emit an alert that breaks through Do Not Disturb and takes over the lock
screen — the "your timer's up" case.

Required for `urgency: "critical"` with `full_takeover: true`. On Android the
frontend prompts the user for the `USE_FULL_SCREEN_INTENT` special-access
permission when a skill declaring this is installed.

Don't declare it for ordinary notifications. It's a loud, intrusive
permission and reviewers will push back.

### `alarm`

Set a device alarm, or open the alarm list, through the platform's own clock
app.

```json
{ "v": 1,
  "alarm": { "op": "set", "hour": 7, "minute": 0, "message": "Wake up" },
  "speak": "Alarm set for 7:00." }
```

The user's Clock app owns scheduling, reboot persistence and ringing — you're
handing off, not implementing an alarm.

### `navigation`

Start navigation to a destination.

```json
{ "v": 1, "navigate": { "destination": "the airport", "mode": "default_app" } }
```

`mode` is `"default_app"` (opens the user's maps app) or `"turn_by_turn"`.

### `media_control`

Emit a `media` action — play a query, or drive transport controls.

```json
{ "v": 1, "media": { "action": "play", "query": "brian eno", "service": "spotify" } }
```

Pair with [`media_services`](#media_services) if you need to know what's
actually installed first.

---

## Requesting something that isn't here

Open an issue before you build. The capability surface is deliberately small
and every addition has to be implementable on every frontend — a capability
only Android can satisfy makes skills that silently don't work elsewhere.

## See also

- [reference-sdk.md](reference-sdk.md) — the SDK functions each capability unlocks
- [reference-actions.md](reference-actions.md) — the envelope slots the frontend capabilities gate
- [troubleshooting.md](troubleshooting.md#capability-and-feature-mismatches) — when the two halves disagree
