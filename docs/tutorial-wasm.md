# Tutorial: a WASM skill

A declarative skill can only say things it already knows. When you need
logic, memory, network access or the device's calendar, you need a WASM skill:
a sandboxed module the engine runs for you.

We're going to build **tally** — a skill that keeps a running count of
something. It'll demonstrate the four things declarative skills can't do:

- persistent state across invocations (`storage_kv`)
- user-configurable settings
- branching on what the user actually said
- returning a rich card instead of a sentence

The finished result is in [`templates/tally`](../templates/tally).

> **Do you actually need WASM?** Only reach for it if you need HTTP, storage,
> a device capability, non-trivial parsing, or a computed response. If your
> skill can be a template string, keep it declarative — it's less to maintain
> and it can't crash.

---

## What you need

```bash
rustup target add wasm32-unknown-unknown
```

And a clone of [ari-engine](https://github.com/ari-digital-assistant/ari-engine)
alongside `ari-skills`, for the validator and the test CLI.

## Step 1 — Copy the template

```bash
cd ari-skills
cp -r templates/tally skills/tally
cd skills/tally
```

Everything below explains what's in there. If you're starting a *different*
skill, this is also the moment to rename: the directory name, the `name:`
field in the manifest, and the `[package] name` in `Cargo.toml`.

## Step 2 — The manifest

The frontmatter is the same shape as a declarative skill, with three
differences. Full field reference:
[reference-manifest.md](reference-manifest.md).

### `wasm` replaces `declarative`

```yaml
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
```

`module` is the filename, relative to the skill directory.
`memory_limit_mb` must be between 1 and 16, and **defaults to 16** if you omit
it. A Rust skill built with `std` needs roughly 1.1 MiB just to start, so
don't drop this below 2 without testing.

Exactly one of `wasm` or `declarative` may be present. Both is an error;
neither is an error.

### `capabilities` declares what you're allowed to touch

```yaml
    capabilities: [storage_kv]
```

This is a hard contract, checked twice:

- **At install time**, the engine rejects the skill if the frontend can't
  provide everything you declared.
- **Also at install time**, a "sneak guard" scans your compiled module's
  imports. Import `storage_get` without declaring `storage_kv` and you're
  rejected. Declare a capability whose functions you never import and you're
  fine — but a reviewer will ask why.

Declare exactly what you use. Every capability, what it grants, and which SDK
feature turns it on: [reference-capabilities.md](reference-capabilities.md).

### `settings` declares the skill's settings screen

```yaml
    settings:
      - key: label
        label: "What are you counting?"
        type: text
        required: false
        default: "things"
        help_text: "Shown on the card, e.g. cups of water."
      - key: goal
        label: "Daily goal"
        type: select
        required: false
        default: "0"
        options:
          - value: "0"
            label: "No goal"
          - value: "5"
            label: "5 a day"
          - value: "8"
            label: "8 a day"
          - value: "10"
            label: "10 a day"
```

The frontend renders this. You read the values back with
`ari::setting_get("<key>")`. Persisted values are always strings — parse them
yourself.

Seven field types exist, including ones that let your skill populate its own
dropdown over the network or run an OAuth sign-in. See
[reference-manifest.md](reference-manifest.md#settings-fields).

### Patterns still do the matching

```yaml
    matching:
      patterns:
        - keywords: [add, tally]
          weight: 0.95
        - keywords: [my, tally]
          weight: 0.95
        - keywords: [reset, tally]
          weight: 0.95
```

**Your WASM module is not called during scoring.** The engine compiles a
native scorer from these patterns at load time. That's what lets the registry
hold hundreds of skills without paying an FFI cost on every utterance.

Notice all three patterns select the same skill. Patterns decide *which skill
runs*; your `execute` decides *what it does*.

(WASM skills can opt into scoring themselves with
`matching.custom_score: true`, which makes the engine call your `score`
export. It's slow and almost never necessary.)

## Step 3 — The code

`src/lib.rs`. Two required exports.

### `score`

```rust
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    0.0
}
```

Required to exist, never called while `custom_score` is false. Return 0.0 and
move on.

### `execute`

```rust
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };

    let (count, spoken) = if input.contains("reset") {
        store(0);
        (0, translate("tally.reset", 0))
    } else if input.contains("add") || input.contains("another") {
        let next = load() + 1;
        store(next);
        (next, translate("tally.added", next))
    } else {
        let now = load();
        (now, translate("tally.current", now))
    };

    ari::respond_action(&envelope(&spoken, count))
}
```

Three things worth knowing:

**The input is already normalised.** You get lowercase, contractions expanded,
punctuation stripped, English number words replaced by digits. Match against
that — `input.contains("don't")` can never be true, because by the time you
see it it's "do not". This applies to `execute`, not just to patterns.

**Every call is a fresh sandbox.** No globals survive between invocations. If
you need memory, that's what `storage_kv` is for.

**You return a packed integer, not a string.** `respond_text` and
`respond_action` do the packing. Don't hand-roll it.

### State

```rust
const KEY_COUNT: &str = "count";

fn load() -> u32 {
    ari::storage_get(KEY_COUNT)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn store(value: u32) {
    if !ari::storage_set(KEY_COUNT, &value.to_string()) {
        ari::log(ari::LogLevel::Warn, "could not persist the tally");
    }
}
```

Storage is a string→string map scoped to your skill id. No other skill can
read or clobber it. It needs `capabilities: [storage_kv]` and the SDK's
`storage` feature.

Don't confuse it with `setting_get`/`setting_set`, which read and write your
skill's *settings* and need no capability at all.

### Settings and the card

```rust
fn envelope(spoken: &str, count: u32) -> String {
    let label = ari::setting_get("label").unwrap_or("things");
    let goal: u32 = ari::setting_get("goal")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let mut stat = p::Stat::new(count.to_string()).caption(label);
    if goal > 0 {
        stat = stat.pill(p::IconText::new(format!("{count} of {goal}")));
    }

    let accent = if goal > 0 && count >= goal {
        p::Accent::Success
    } else {
        p::Accent::Default
    };

    p::Envelope::new()
        .speak(spoken)
        .card(
            p::Card::new("tally")
                .title(translate_plain("tally.title"))
                .accent(accent)
                .stat(stat),
        )
        .to_json()
}
```

`speak` is what Ari says out loud. The card is what appears in the chat.

**The card id is stable on purpose.** Re-emitting a card with an id that's
already on screen *replaces* it. Use a random id and every utterance stacks
another card up the screen.

Full envelope vocabulary — alerts, notifications, countdowns, buttons,
follow-up questions: [reference-actions.md](reference-actions.md).

### Strings

```rust
fn translate(key: &str, count: u32) -> String {
    let count = count.to_string();
    ari::t(key, &[("count", count.as_str())])
        .unwrap_or(key)
        .to_string()
}
```

`ari::t` looks the key up in `strings/<user's locale>.json`, falls back to
English, and substitutes `{placeholders}`. It needs no capability.

`strings/en.json`:

```json
{
  "tally.title": "Tally",
  "tally.added": "That's {count}.",
  "tally.current": "You're on {count}.",
  "tally.reset": "Tally reset."
}
```

Routing everything through `t()` from the start costs nothing and means
adding a language later is a new JSON file, not a refactor.

## Step 4 — `Cargo.toml`

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
ari-skill-sdk = { path = "../../sdk/rust", features = ["storage"] }

[profile.release]
opt-level = "s"
lto = true
strip = true
```

`crate-type = ["cdylib"]` is mandatory — an rlib produces no WASM module.

**SDK features and manifest capabilities must agree.** `features = ["storage"]`
gives you `storage_get`/`storage_set` in Rust; `capabilities: [storage_kv]`
gives you permission to import them. Get one without the other and you fail at
compile time or install time respectively.

The release profile isn't decoration: skills are downloaded over mobile data
and bundles are capped at 8 MiB.

## Step 5 — Build

```bash
./build.sh
```

```
wrote skill.wasm (35528 bytes)
```

That's `cargo build --target wasm32-unknown-unknown --release` plus a copy of
the artifact to `skill.wasm`.

## Step 6 — Validate and test

```bash
cd ../..
./tools/validate skills/tally
```

```
✓ skills/tally: com.example.tally (8 examples)

validated 1 skill(s), 0 failure(s)
```

The validator compiles your module and runs the sneak guard, so this catches
capability mismatches — not just manifest typos.

Now run it. `storage_kv` isn't granted by default, so ask for it:

```bash
cd ../ari-engine
cargo run -p ari-cli -- \
  --extra-skill-dir ../ari-skills/skills/tally \
  --host-capabilities storage_kv \
  --storage-dir /tmp/tally-test \
  "add one to my tally"
```

```json
{
  "cards": [
    {
      "id": "tally",
      "stat": { "caption": "things", "headline": "1" },
      "title": "Tally"
    }
  ],
  "speak": "That's 1.",
  "v": 1
}
```

Run it again and the headline becomes `2` — that's `storage_kv` persisting
across separate processes. Then `"reset my tally"` puts it back to zero.

Omit `--storage-dir` and you get a temp directory, which is fine for a quick
check but won't survive a reboot.

### Logging

```rust
ari::log(ari::LogLevel::Info, "something happened");
```

On the CLI these go to stderr. On Android they surface under the `AriSkill`
logcat tag with your skill id prepended:

```bash
adb logcat -s AriSkill
```

### On a device

The CLI prints the envelope JSON. It can't show you whether the card looks
right, whether TTS reads it naturally, or whether an alert actually rings.

```bash
./tools/sideload-android skills/tally
```

Do this before opening a PR for any skill that emits cards, alerts or
notifications. See [publishing.md](publishing.md#test-on-a-device).

## Step 7 — Submit

Same as any skill: [publishing.md](publishing.md).

---

## Where next

| You want to… | Read |
|---|---|
| Call an HTTP API | [reference-capabilities.md](reference-capabilities.md#http) |
| Read the device's location, calendar or tasks | [reference-capabilities.md](reference-capabilities.md) |
| Ask the user a follow-up question | [reference-actions.md](reference-actions.md#await_reply--asking-a-follow-up) |
| Fill a settings dropdown from your own server | [reference-manifest.md](reference-manifest.md#dynamic_select) |
| Add an OAuth "Sign in" button | [reference-manifest.md](reference-manifest.md#oauth-sign-in) |
| Look up an SDK function | [reference-sdk.md](reference-sdk.md) |
| Ship a second language | [i18n.md](i18n.md) |
