# Writing WASM Skills

WASM skills run sandboxed code inside Ari's engine. Use them when declarative skills aren't enough — HTTP calls, persistent state, non-trivial parsing.

Two SDKs are available: **Rust** and **AssemblyScript**. Both produce `.wasm` modules that conform to the Ari WASM ABI described below.

## Prerequisites

**Rust:**
```bash
# Install the WASM target
rustup target add wasm32-unknown-unknown
```

**AssemblyScript:**
```bash
# Node.js 18+ required
node --version
```

## Quick start

### Rust

```bash
# Copy the template
cp -r templates/rust my-cool-skill

# Edit the skill
cd my-cool-skill
# 1. Update SKILL.en.md — set id, name, description, keywords
# 2. Edit src/lib.rs — implement your score/execute logic
# 3. Update Cargo.toml — rename the package, enable features if needed

# Build
./build.sh
# Produces skill.wasm

# Test against a local engine
cd ../../ari-engine
cargo run -p ari-cli -- --extra-skill-dir ../ari-skills/my-cool-skill "your test input"

# Or sideload onto a connected Android device / emulator for end-to-end
# testing against the real chat UI, TTS, and action-envelope renderer.
# This is the fastest way to exercise assets, alerts, and notifications
# — anything the CLI can't render.
./tools/sideload-android skills/my-cool-skill
```

### AssemblyScript

```bash
# Copy the template
cp -r templates/assemblyscript my-cool-skill

# Edit the skill
cd my-cool-skill
# 1. Update SKILL.en.md — set id, name, description, keywords
# 2. Edit assembly/index.ts — implement your score/execute logic

# Build
./build.sh
# Produces skill.wasm

# Test against a local engine
cd ../../ari-engine
cargo run -p ari-cli -- --extra-skill-dir ../ari-skills/my-cool-skill "your test input"

# Or sideload onto a connected Android device / emulator for end-to-end
# testing against the real chat UI, TTS, and action-envelope renderer.
./tools/sideload-android skills/my-cool-skill
```

## SDK API reference

### Rust (`ari-skill-sdk`)

Add to your `Cargo.toml`:
```toml
[dependencies]
ari-skill-sdk = { path = "../../sdk/rust" }
```

Enable features for host imports your skill needs:
```toml
ari-skill-sdk = { path = "../../sdk/rust", features = ["http"] }
ari-skill-sdk = { path = "../../sdk/rust", features = ["storage"] }
ari-skill-sdk = { path = "../../sdk/rust", features = ["location"] }
ari-skill-sdk = { path = "../../sdk/rust", features = ["http", "storage"] }
```

#### Core functions

```rust
use ari_skill_sdk as ari;

// Read input from the host (call inside score/execute)
let text: &str = unsafe { ari::input(ptr, len) };

// Pack a text response for return from execute()
let packed: i64 = ari::respond_text("Hello!");

// Or a structured action envelope — JSON parsed into Response::Action host-side.
// Easiest path: build it via the typed presentation module (default-on
// feature). See docs/action-responses.md for the full envelope vocabulary.
use ari_skill_sdk::presentation as p;
let envelope = p::Envelope::new()
    .speak("Opening Spotify.")
    .launch_app("Spotify")
    .to_json();
let packed: i64 = ari::respond_action(&envelope);

// Logging (levels: Trace, Debug, Info, Warn, Error).
// On Android, lines surface under logcat tag `AriSkill` with the skill
// id prepended: `adb logcat -s AriSkill`. On the CLI engine they go
// through the host's configured sink (stderr by default).
ari::log(ari::LogLevel::Info, "something happened");

// Wall-clock time in milliseconds, and 64 bits of entropy. Both unconditional.
let now: i64 = ari::now_ms();
let seed: u64 = ari::rand_u64();

// Check if a capability is available
if ari::has_capability("http") { /* ... */ }
```

#### HTTP (`features = ["http"]`)

```rust
let resp = ari::http_fetch("https://api.example.com/data");
// resp.status: u16 (HTTP status code, 0 on network error)
// resp.body: Option<&str>
// resp.error: Option<&str> (only on network/timeout failures)
```

#### Storage (`features = ["storage"]`)

```rust
// Read
if let Some(value) = ari::storage_get("my_key") {
    // use value
}

// Write (returns true on success)
ari::storage_set("my_key", "my_value");
```

#### Location (`features = ["location"]`)

Coarse device location. Declare `capabilities: [location]` and enable the
`location` SDK feature.

```rust
use ari_skill_sdk as ari;

let loc = ari::location();
match loc.status {
    ari::LocationStatus::Ok => {
        // loc.lat, loc.lon, loc.accuracy_m, loc.timestamp_ms
    }
    ari::LocationStatus::PermissionDenied => { /* ask the user to grant location */ }
    ari::LocationStatus::Unavailable | ari::LocationStatus::Timeout => { /* fall back */ }
}
```

Use `ari::location_with(max_age_ms, timeout_ms)` to override the defaults
(10 min cached-fix freshness / 5 s active-fix timeout). Coarse accuracy only —
fine location is never requested. On hosts without a location implementation
(CLI, Linux until its app exists) the status is always `Unavailable`.

#### Interactive settings (`features = ["settings"]`)

Helpers for the optional `settings_query` export — populating a `dynamic_select`
dropdown or answering a `validate` field from inside the WASM sandbox. Pure
(no WASM ABI, no host imports), so they're natively unit-testable.

```rust
use ari::settings::{parse_query_input, SelectOpt, SettingsResult};

// Parse the host's {field, values} payload.
let q = parse_query_input(input).unwrap();        // -> SettingsQueryInput
let field = q.field.as_str();                     // which field is being queried
let url = q.value("base_url");                     // Option<&str> — a depends_on sibling

// Build the {ok, error?, options?, message?} reply, then .to_json().
SettingsResult::options(vec![SelectOpt { value: "x".into(), label: "X".into() }]).to_json();
SettingsResult::validated("Connected").to_json();  // green ✓
SettingsResult::error("bad token").to_json();      // red ✗
```

Full author guide, including the manifest field declarations and a worked
example: [skill-authors.md](skill-authors.md#server-backed-settings).

#### OAuth sign-in (`features = ["authorize"]`)

For skills that need to authenticate the user against a third-party service
(OAuth 2.0 / IndieAuth), `ari::authorize` opens the system browser at an
authorization URL, waits for the redirect back to your `redirect_uri`, and
hands you the callback query parameters. Your skill builds the URL and owns the
protocol (state, PKCE, exchanging the code for a token); the host only drives
the browser round-trip.

```rust
// Build your provider's authorization URL (with client_id, redirect_uri,
// state, PKCE challenge, etc.), then hand it to the host.
let redirect = ari::oauth_redirect_uri();
let res = ari::authorize(&auth_url, &redirect, 300_000);
if res.ok {
    let code = res.get("code").unwrap_or("");
    let state = res.get("state").unwrap_or("");
    // verify state, then POST `code` to the token endpoint via ari::http_request
} else {
    // res.error is one of: "cancelled", "timeout", "no_browser",
    // "mismatch", "bad_request", "bad_response"
}
```

- `authorize(auth_url, redirect_uri, timeout_ms) -> AuthorizeResult` — `timeout_ms`
  is the max time to wait for the redirect (300_000 = 5 min is typical).
- `AuthorizeResult { ok: bool, params: Vec<(String, String)>, error: Option<String> }`
  — `res.get("code")` / `res.get("state")` look up a returned param by key.
- The `redirect_uri` you pass is also matched by the host against the actual
  callback; a mismatch fails with `error = "mismatch"`.

Call `ari::oauth_redirect_uri()` to get the redirect URI to build into your
auth URL and pass to `authorize` — never hardcode it. On Android it's the
verified App Link `https://heyari.dev/oauth/callback`; the same call returns the
right value on other frontends, so your skill code is portable.

PKCE helpers (`sha256`, `base64url_nopad`) live behind the SDK's `crypto`
feature. Full worked example (PKCE, the `action` settings button that triggers
it, and the code-for-token exchange):
[skill-authors.md](skill-authors.md#action-buttons-and-oauth-sign-in).

#### Skill entry points

Your crate must export two functions:

```rust
#[no_mangle]
pub extern "C" fn score(ptr: i32, len: i32) -> f32 {
    // Return 0.0-1.0. For most skills, leave at 0.0 and let
    // the manifest keywords handle scoring (custom_score: false).
    0.0
}

#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    ari::respond_text("your response here")
}
```

The SDK automatically exports `memory` and `ari_alloc` — you don't need to handle those. A skill with server-backed settings can export two more functions: `settings_query` (drive a dropdown or validate a field) and `settings_action` (handle a settings button press, e.g. OAuth sign-in). See the optional-exports table above and the [interactive settings](#interactive-settings-features--settings) helpers.

### AssemblyScript (`ari-skill-sdk-as`)

Import the SDK:
```typescript
import { ari_alloc, input, respond, log, INFO } from "ari-skill-sdk-as/assembly";
```

**You must re-export `ari_alloc`** so the host can find it:
```typescript
export { ari_alloc };
```

#### Core functions

```typescript
// Read input from the host
const text: string = input(ptr, len);

// Pack a response string
const packed: i64 = respond("Hello!");

// Logging (levels: TRACE=0, DEBUG=1, INFO=2, WARN=3, ERROR=4)
log(INFO, "something happened");

// Check capabilities
import { hasCapability } from "ari-skill-sdk-as/assembly";
if (hasCapability("http")) { /* ... */ }
```

#### HTTP

```typescript
import { httpFetchRaw } from "ari-skill-sdk-as/assembly/http";

const json: string | null = httpFetchRaw("https://api.example.com/data");
// Returns raw JSON: {"status": 200, "body": "..."} or null
```

#### Storage

```typescript
import { storageGet, storageSet } from "ari-skill-sdk-as/assembly/storage";

const value: string | null = storageGet("my_key");
const ok: bool = storageSet("my_key", "my_value");
```

#### Skill entry points

```typescript
export function score(ptr: i32, len: i32): f32 {
  return 0.95;
}

export function execute(ptr: i32, len: i32): i64 {
  const text = input(ptr, len);
  return respond("your response");
}
```

#### Build flags

AS skills must be compiled with `--use abort=` to prevent importing `env::abort`, which the Ari host doesn't provide. The template's `build.sh` includes this.

## ABI contract

For authors who want to understand what the SDK does under the hood.

### Required exports

| Export | Signature | Purpose |
|--------|-----------|---------|
| `memory` | linear memory | Host reads/writes input and responses here |
| `ari_alloc` | `(size: i32) -> i32` | Host calls this to allocate space for input strings and import responses |
| `score` | `(ptr: i32, len: i32) -> f32` | Return relevance score in [0.0, 1.0] for the UTF-8 input at (ptr, len) |
| `execute` | `(ptr: i32, len: i32) -> i64` | Process input at (ptr, len), return the tagged response value described below |

Optional exports:

| Export | Signature | Purpose |
|--------|-----------|---------|
| `settings_query` | `(ptr: i32, len: i32) -> i64` | Drive a `dynamic_select` dropdown or a `validate` field at settings-time. Input is `{field, values}` JSON, output is `{ok, error?, options?, message?}` JSON — same ABI as `execute`. Reuses the `http`/`t` imports; needs the SDK `settings` feature. Full guide: [skill-authors.md](skill-authors.md#server-backed-settings) |
| `settings_action` | `(ptr: i32, len: i32) -> i64` | Handle a settings-screen `action` button press (e.g. an OAuth "Sign in" button). Input is `{action, values}` JSON, output is `{ok, error?, message?}` JSON — same ABI as `execute`. Full guide: [skill-authors.md](skill-authors.md#action-buttons-and-oauth-sign-in) |

### Tagged `execute` return value

`execute` packs a tag byte + pointer + length into the 64-bit return:

```text
bits 63..56 → tag  (0x00 = Text, 0x01 = Action, 0x02 reserved for Binary)
bits 55..32 → ptr  (24-bit pointer — caps skill memory at 16 MiB, enforced)
bits 31..0  → len  (32-bit byte length of the payload)
```

- `tag = 0x00` → host treats payload as UTF-8 and wraps in `Response::Text`
- `tag = 0x01` → host parses payload as JSON and wraps in `Response::Action`

The Rust SDK helpers (`respond_text`, `respond_action`) do the packing for you.
If you hand-roll the WAT, remember that a tag byte in `bits 63..56` of zero is
automatic for any small pointer — this is why existing text-only skills keep
working without changes.

Any tag ≥ `0x02` is currently a contract violation; the host falls back to
`(skill error)`. New tags will be added via engine releases, not silently.

### Host imports (all in the `ari` module)

Unconditional — any skill may import these without declaring a capability:

| Import | Signature |
|--------|-----------|
| `log` | `(level: i32, ptr: i32, len: i32)` |
| `get_capability` | `(name_ptr: i32, name_len: i32) -> i32` |
| `now_ms` | `() -> i64` — current Unix time in milliseconds |
| `rand_u64` | `() -> i64` — 64 bits of cryptographically-random entropy |
| `oauth_redirect_uri` | `() -> i64` — the host's canonical OAuth redirect URI (packed string) |
| `setting_get` | `(key_ptr: i32, key_len: i32) -> i64` — read one of the skill's own settings |
| `setting_set` | `(key_ptr: i32, key_len: i32, val_ptr: i32, val_len: i32) -> i32` — write one of the skill's own settings |
| `get_locale`, `t`, `format_date`, `format_number`, `format_currency` | i18n helpers — see [i18n.md](i18n.md) |

(`setting_get`/`setting_set` are ungated because they only touch the skill's
*own* settings store — distinct from the capability-gated `storage_get`/`storage_set`
scratch store. There are a few more unconditional time/locale imports —
`local_now_components`, `local_timezone_id`, `args` — exposed through the SDK's
i18n and args helpers.)

Capability-gated:

| Import | Signature | Capability |
|--------|-----------|------------|
| `http_fetch` | `(url_ptr: i32, url_len: i32) -> i64` | `http` |
| `http_request` | `(req_ptr: i32, req_len: i32) -> i64` | `http` |
| `storage_get` | `(key_ptr: i32, key_len: i32) -> i64` | `storage_kv` |
| `storage_set` | `(key_ptr: i32, key_len: i32, val_ptr: i32, val_len: i32) -> i32` | `storage_kv` |
| `authorize` | `(req_ptr: i32, req_len: i32) -> i64` | `authorize` |
| `location_current` | `(max_age_ms: i64, timeout_ms: i64) -> i64` | `location` |

The `tasks_*` and `calendar_*` host imports (gated by the `tasks` / `calendar`
capabilities) follow the same pattern; they're surfaced through dedicated SDK
modules rather than raw imports.

### Sandbox limits

- **Memory**: default 16 MiB, configurable via `metadata.ari.wasm.memory_limit_mb` (must be 1..=16, enforced by the 24-bit ptr field in the `execute` return)
- **Fuel**: 50,000,000 units per call (~tens of milliseconds)
- **Isolation**: fresh store per call — no state survives between invocations (use `storage_kv` for persistence)

## Capabilities

Declare capabilities in your `SKILL.en.md`:
```yaml
    capabilities: [http, storage_kv]
```

Only import the SDK modules you need. The WASM module's imports must match the declared capabilities — the host's sneak guard rejects any module that imports `http_fetch` without declaring `[http]`, and vice versa.

Available capabilities: `http`, `storage_kv`, `authorize`, `notifications`, `launch_app`, `clipboard`, `tts`, `location`, `calendar`, `tasks`.

## Common pitfalls

1. **Forgot to re-export `ari_alloc` (AssemblyScript).** Your skill compiles but the host can't write input. Add `export { ari_alloc };` in your index.ts.

2. **`env::abort` import (AssemblyScript).** Compile with `--use abort=` to prevent this. Without it, the WASM module imports `env::abort` which the host doesn't provide.

3. **Memory limit too low.** Rust cdylib skills with std need ~1.1 MiB initial memory. Set `memory_limit_mb: 4` in SKILL.en.md. The hand-written WAT skills use 1 MiB, but compiled Rust skills are larger.

4. **Feature not enabled (Rust).** If you call `http_fetch` without `features = ["http"]`, you'll get a compile error. If you enable the feature but don't declare `capabilities: [http]` in SKILL.en.md, the host will reject the skill.

5. **Directory name must match `name` in SKILL.en.md.** The skill directory name and the `name:` field in the YAML frontmatter must be identical. This is an AgentSkills rule.

6. **Unused host imports are stripped by LTO (Rust).** If you enable a feature but don't call the function, LTO strips the import — the capability check still passes because the module never imports it. This is correct behaviour.

## Testing locally

```bash
cd ari-engine
cargo run -p ari-cli -- --extra-skill-dir /path/to/your/skill "test input"
```

For skills that use HTTP or storage, pass the host capabilities flag:
```bash
cargo run -p ari-cli -- \
  --extra-skill-dir /path/to/your/skill \
  --host-capabilities http,storage_kv \
  --storage-dir /tmp/skill-storage \
  "test input"
```

## Internationalisation

Five SDK helpers cover the i18n surface — `ari::get_locale`, `ari::t`,
`ari::format_date`, `ari::format_number`, `ari::format_currency`. All
ungated. See **[i18n.md](i18n.md)** for the contract, the
`SKILL.{locale}.md` + `strings/{locale}.json` layout, and worked
declarative + WASM examples.
