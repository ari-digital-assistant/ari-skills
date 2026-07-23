# Rust SDK reference

`ari-skill-sdk` wraps the WASM host imports in ordinary Rust. This page is the
API surface and, at the bottom, the raw ABI underneath it.

Using AssemblyScript instead? [assemblyscript.md](assemblyscript.md).

```toml
[dependencies]
ari-skill-sdk = { path = "../../sdk/rust", features = ["http"] }
```

## Feature flags

| Feature | Default | Unlocks | Needs capability |
|---|---|---|---|
| `std` | ✅ | The standard library | — |
| `presentation` | ✅ | [Action envelope builders](reference-actions.md), `parse_reply` | — |
| `settings` | ✅ | `settings_query` / `settings_action` helpers | — |
| `http` | | `http_fetch`, `http_request` | `http` |
| `storage` | | `storage_get`, `storage_set` | `storage_kv` |
| `location` | | `location`, `location_with` | `location` |
| `authorize` | | `authorize` | `authorize` |
| `crypto` | | `sha256`, `base64url_nopad` (PKCE) | — |
| `tasks` | | `tasks_*` | `tasks` |
| `calendar` | | `calendar_*` | `calendar` |
| `media` | | `media_services` | `media_services` |
| `clock` | | `local_now_components`, `local_timezone_id` | — |

Two rules that catch everyone:

- **A feature without its capability fails at install.** The sneak guard scans
  your module's imports and rejects anything you didn't declare.
- **A capability without its feature just doesn't compile.** The function
  isn't there.

`crypto` and `clock` are ungated — they're feature flags purely so a
text-only skill can ship a leaner module.

Shipping something tiny? `default-features = false` drops the JSON machinery.

---

## Always available

No capability, no feature flag.

### Input and output

```rust
// Read the host's input. Call inside score/execute.
let text: &str = unsafe { ari::input(ptr, len) };

// Return a plain string.
ari::respond_text("Hello!")

// Return an action envelope (JSON).
ari::respond_action(&json)
```

`respond(...)` is deprecated — use `respond_text`.

The input you receive is **normalised**: lowercased, contractions expanded,
punctuation stripped, English number words turned into digits. See
[reference-manifest.md](reference-manifest.md#input-normalisation).

### Logging

```rust
ari::log(ari::LogLevel::Info, "something happened");
// Trace | Debug | Info | Warn | Error
```

On Android these appear under the `AriSkill` logcat tag with your skill id
prepended:

```bash
adb logcat -s AriSkill
```

On the CLI they go to stderr.

### Time and entropy

```rust
let now: i64 = ari::now_ms();     // UTC epoch milliseconds
let seed: u64 = ari::rand_u64();  // cryptographically random
```

With `features = ["clock"]` you also get local wall-clock time — which is what
you want for anything a human will read:

```rust
let c = ari::local_now_components();  // .year .month .day .hour .minute .second …
let tz: String = ari::local_timezone_id();
```

### Capability checks

```rust
if ari::has_capability("http") { /* … */ }
```

Useful when a capability is optional to your skill and you want to degrade
gracefully rather than declare something you rarely use.

### Your own settings

```rust
let value: Option<&str> = ari::setting_get("base_url");
let ok: bool = ari::setting_set("token", "…");
```

**Ungated** — these touch only your own settings, declared in
`metadata.ari.settings`. Don't confuse them with `storage_get`/`storage_set`,
which are a separate scratch store behind the `storage_kv` capability.

Values are always strings. Writes to a field declared `type: secret` are
routed to encrypted storage automatically.

### Router arguments

```rust
let args: Option<&str> = ari::args();   // JSON, or None
```

When the router dispatches your skill it can pass the arguments it extracted,
matching the `args` you declared in `metadata.ari.examples`. `None` means you
were selected by the keyword matcher instead — so always have a fallback path
that parses the raw utterance.

### Internationalisation

```rust
let s = ari::t("greet.hello", &[("name", "Keith")]).unwrap_or("Hello!");
let d = ari::format_date(ts_ms, "", "long").unwrap_or("today");
let n = ari::format_number(1234.56, "", "").unwrap_or("1234.56");
let m = ari::format_currency(1234.56, "EUR", "").unwrap_or("");
let lang: &str = ari::get_locale();   // "en", "it", …
```

Pass `""` as the locale to use the user's active language. Full contract:
[i18n.md](i18n.md).

### Multi-turn replies

```rust
pub struct Reply { pub context: String, pub text: String }

if let Some(reply) = ari::parse_reply(input) { … }
```

Behind the `presentation` feature. See
[reference-actions.md](reference-actions.md#await_reply--asking-a-follow-up).

---

## Capability-gated

Each of these needs both its SDK feature and its manifest capability. The
signatures and usage notes live in
[reference-capabilities.md](reference-capabilities.md) so there's one place to
keep current:

[`http_fetch` / `http_request`](reference-capabilities.md#http) ·
[`storage_get` / `storage_set`](reference-capabilities.md#storage_kv) ·
[`location` / `location_with`](reference-capabilities.md#location) ·
[`authorize` / `oauth_redirect_uri`](reference-capabilities.md#authorize) ·
[`media_services`](reference-capabilities.md#media_services) ·
[`tasks_*`](reference-capabilities.md#tasks) ·
[`calendar_*`](reference-capabilities.md#calendar)

## Settings helpers (`features = ["settings"]`)

Pure functions — no WASM ABI, so you can unit-test your settings logic on the
host.

```rust
use ari::settings::{parse_query_input, parse_action_input, SelectOpt, SettingsResult};

let q = parse_query_input(input)?;      // .field, .value("dep_key")
let a = parse_action_input(input)?;     // .action, .value("dep_key")

SettingsResult::options(vec![SelectOpt { value: "x".into(), label: "X".into() }]).to_json();
SettingsResult::validated("Connected").to_json();   // green ✓
SettingsResult::error("Bad token").to_json();       // red ✗
SettingsResult::validated("Signed in.").with_refresh().to_json();
```

Full guide: [reference-manifest.md](reference-manifest.md#settings-fields).

## Presentation builders (`features = ["presentation"]`)

```rust
use ari_skill_sdk::presentation as p;

p::Envelope::new()
    .speak("…")
    .card(p::Card::new("id").title("…").stat(p::Stat::new("7")))
    .alert(p::Alert::new("id").title("…").urgency(p::Urgency::Critical))
    .notification(p::Notification::new("id").title("…"))
    .launch_app("Spotify")
    .search("…")
    .open_url("https://…")
    .clipboard("…")
    .alarm(p::Alarm::set(7, 0))
    .navigate(p::Navigate::to("the airport"))
    .await_reply("…")
    .dismiss_card("id")
    .to_json();
```

Types: `Envelope` `Card` `Stat` `ListCard` `ListRow` `IconText` `Alert`
`Notification` `Action` `OnComplete` `Alarm` `Navigate` `Asset` `Day`, plus
the enums `Accent` `Urgency` `Sound` `Importance` `Style`.

Field-by-field semantics: [reference-actions.md](reference-actions.md).

> No builder exists for the `media` slot. Hand-build that JSON.

---

## The ABI

You don't need this to write a skill. It's here for people porting a new
language or debugging something strange.

### Required exports

| Export | Signature | Purpose |
|---|---|---|
| `memory` | linear memory | Where the host reads and writes |
| `ari_alloc` | `(size: i32) -> i32` | Host calls this to allocate for input and import results |
| `score` | `(ptr: i32, len: i32) -> f32` | Relevance in [0.0, 1.0]. Only called when `custom_score: true` |
| `execute` | `(ptr: i32, len: i32) -> i64` | Handle the input; return a packed response |

The Rust SDK exports `memory` and `ari_alloc` for you.

### Optional exports

| Export | Signature | Purpose |
|---|---|---|
| `settings_query` | `(ptr: i32, len: i32) -> i64` | Fill a `dynamic_select`, or answer a `validate` field |
| `settings_action` | `(ptr: i32, len: i32) -> i64` | Handle a settings `action` button press |

Both take `{field\|action, values}` JSON and return
`{ok, error?, options?, message?}` JSON, packed the same way as `execute`.

### The packed return value

`execute` packs a tag, a pointer and a length into its `i64`:

```
bits 63..56 → tag  (0x00 = Text, 0x01 = Action, 0x02 reserved)
bits 55..32 → ptr  (24-bit — this is what caps skill memory at 16 MiB)
bits 31..0  → len  (byte length of the payload)
```

- `0x00` — payload is UTF-8 text.
- `0x01` — payload is JSON, parsed into an action envelope.
- Anything ≥ `0x02` is a contract violation; the host returns `(skill error)`.

A zero top byte is automatic for any small pointer, which is why text-only
skills written before the tagged ABI still work.

### Host imports (module `ari`)

Ungated — import freely:

| Import | Signature |
|---|---|
| `log` | `(level: i32, ptr: i32, len: i32)` |
| `get_capability` | `(name_ptr: i32, name_len: i32) -> i32` |
| `now_ms` | `() -> i64` |
| `rand_u64` | `() -> i64` |
| `local_now_components` | `() -> i64` (packed JSON) |
| `local_timezone_id` | `() -> i64` (packed string) |
| `setting_get` | `(key_ptr: i32, key_len: i32) -> i64` |
| `setting_set` | `(key_ptr, key_len, val_ptr, val_len: i32) -> i32` |
| `args` | `() -> i64` (packed JSON) |
| `oauth_redirect_uri` | `() -> i64` (packed string) |
| `get_locale`, `t`, `format_date`, `format_number`, `format_currency` | i18n — see [i18n.md](i18n.md) |

Capability-gated:

| Import | Capability |
|---|---|
| `http_fetch`, `http_request` | `http` |
| `storage_get`, `storage_set` | `storage_kv` |
| `location_current` | `location` |
| `authorize` | `authorize` |
| `media_services` | `media_services` |
| `tasks_provider_installed`, `tasks_list_lists`, `tasks_insert`, `tasks_delete`, `tasks_query_in_range` | `tasks` |
| `calendar_has_write_permission`, `calendar_list_calendars`, `calendar_insert`, `calendar_delete`, `calendar_query_in_range` | `calendar` |

Importing a gated function without declaring its capability is rejected at
install time. Values returned as `-> i64` are a packed `(ptr << 32) | len`;
zero means "nothing".

### Sandbox limits

| Limit | Value |
|---|---|
| Memory | `wasm.memory_limit_mb`, default **16**, range 1–16 |
| Fuel | 50,000,000 units per call (tens of milliseconds) |
| Isolation | A fresh store per call — nothing survives between invocations |
| Bundle size | 8 MiB for the whole skill directory |

## See also

- [reference-capabilities.md](reference-capabilities.md)
- [reference-actions.md](reference-actions.md)
- [troubleshooting.md](troubleshooting.md)
