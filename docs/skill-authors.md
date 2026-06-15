# Writing an Ari Skill

A short, practical guide. For the full architecture, see [skill-system.md](skill-system.md).

## What you need to know first

- A skill is a folder containing a `SKILL.en.md` manifest (localized variants are `SKILL.{locale}.md`, e.g. `SKILL.it.md`). Optionally a `strings/` dir for i18n, a WASM module, an icon, and reference docs. See [i18n.md](i18n.md) for the multi-language layout.
- Skills come in two flavours you can author: **declarative** (just a manifest, no code) and **WASM** (a sandboxed module). Most skills should be declarative.
- The format is [AgentSkills](https://agentskills.io)-compatible. Ari-specific config lives under `metadata.ari.*` in the YAML frontmatter.
- You contribute by opening a pull request against [ari-digital-assistant/ari-skills](https://github.com/ari-digital-assistant/ari-skills). Maintainers review, CI signs, the registry publishes.
- You don't need a Rust toolchain unless you're writing a WASM skill.

## Walkthrough: a declarative coin-flip skill

Let's build a skill that responds to "flip a coin" with "Heads." or "Tails."

### 1. Fork and clone

Fork [ari-digital-assistant/ari-skills](https://github.com/ari-digital-assistant/ari-skills) on GitHub, then:

```bash
git clone git@github.com:<your-user>/ari-skills.git
cd ari-skills
```

### 2. Create the skill directory

The directory name must match the AgentSkills `name` field — lowercase letters, digits, and hyphens only.

```bash
mkdir -p skills/coin-flip
```

### 3. Write `SKILL.en.md`

Create `skills/coin-flip/SKILL.en.md` (the canonical English manifest — localized variants are `SKILL.{locale}.md`, e.g. `SKILL.it.md`; see [i18n.md](i18n.md)):

```markdown
---
name: coin-flip
description: Flips a virtual coin and returns heads or tails. Use when the user asks to flip a coin, toss a coin, or make a random binary choice.
license: MIT
metadata:
  ari:
    id: ai.example.coinflip
    version: "0.1.0"
    author: Your Name <you@example.com>
    engine: ">=0.3,<0.4"
    capabilities: []
    languages: [en]
    specificity: high
    matching:
      patterns:
        - keywords: [flip, coin]
          weight: 0.95
        - keywords: [toss, coin]
          weight: 0.95
    examples:
      - text: "flip a coin"
      - text: "toss a coin"
      - text: "heads or tails"
      - text: "can you flip a coin for me"
      - text: "let's leave it to chance"
    declarative:
      response_pick: ["Heads.", "Tails."]
---

# Coin Flip

Flips a virtual coin. Example: "flip a coin" → "Heads."
```

A few things worth pointing at:

- **`name` and the directory name must match.** AgentSkills rule. Lowercase, hyphens, no leading/trailing/consecutive hyphens.
- **`description` is two sentences for a reason.** First sentence: what the skill does. Second sentence: when to use it, with semantic keywords. The FunctionGemma router reads this to decide whether to activate the skill. A vague description means a skill the router will never pick.
- **`metadata.ari.id` is reverse-DNS.** This is the unique registry identifier. Pick a domain you control. The directory `name` is just a slug.
- **`specificity: high`** means "I'm confident about a narrow input." A coin-flip skill with `keywords: [flip, coin]` should never fire on "flip the pancakes" — high specificity + tight keywords prevents that.
- **`capabilities: []`** because we don't need network, location, storage, or anything else. Skills that don't need capabilities should declare an empty list, not omit the field.
- **`response_pick`** picks one entry at random per invocation. Use `response` for a fixed string, `response_pick` for randomness.

### 4. Add an icon (optional)

```bash
mkdir skills/coin-flip/assets
cp ~/Downloads/coin-icon.png skills/coin-flip/assets/icon.png
```

96×96 PNG is the recommendation. The frontend scales it.

### 5. Validate locally

```bash
./tools/validate skills/coin-flip
```

This is the same check the registry CI runs. It parses the frontmatter, checks AgentSkills naming rules, confirms `metadata.ari.id` doesn't collide with anything in `index.json`, verifies exactly one of `declarative`/`wasm` is present, and lints the matching patterns for obvious mistakes.

### 6. Test your skill locally

You've got two complementary options. Use both — one tells you *what* matched, the other tells you how the skill actually *feels* in the app.

#### With the CLI engine (fastest)

If you have the Ari engine checked out:

```bash
cd ../ari/ari-engine
cargo run -p ari-cli -- \
  --extra-skill-dir ../../ari-skills/skills/coin-flip \
  "flip a coin"
```

You should see `Heads.` or `Tails.` Try a few inputs that *shouldn't* match — `"what time is it"`, `"flip the pancakes"` — and confirm your skill stays out of it. This is the right loop for iterating on keyword patterns and scoring — it's deterministic, instant, and prints the winning skill.

#### With the Android app (end-to-end — recommended before opening a PR)

Everything CLI testing can't tell you — whether TTS says the right thing, whether your action envelope renders correctly, whether a declarative `launch_app` target resolves on a real device, whether FunctionGemma routes ambiguous paraphrases — needs the real frontend. The sideload tool pushes your working tree straight into the app's private skills dir on a connected device or emulator, no PR flow required:

```bash
./tools/sideload-android skills/coin-flip
```

Under the hood: rebuild (if `build.sh` exists) → validate → push via `adb` + `run-as` → force-stop and relaunch the app so the engine re-scans on startup. It pushes every manifest present (`SKILL.en.md`, any `SKILL.{locale}.md`, and a legacy bare `SKILL.md`), the `strings/` translation tables, `skill.wasm` if present, and `assets/`. A few seconds per iteration.

The edit-sideload-test loop is worth using for:

- **Development** — iterate on code, WASM builds, `SKILL.{locale}.md` content and `strings/` translations. Faster than any install flow.
- **Debugging behaviour** — confirm the skill is actually being picked up, watch `adb logcat` for engine load messages, reproduce issues that only show up with the real STT/TTS path. For WASM skills, anything your skill emits via `ari::log(...)` in the SDK surfaces under the `AriSkill` tag with the skill id prepended — `adb logcat -s AriSkill` shows every skill, grep by skill id to narrow down.
- **Tuning your description and examples** — the `description` and `examples` fields in `metadata.ari` are what FunctionGemma routes on. Sideload the skill and try the paraphrases you wrote in `examples` as actual utterances. If they don't route to your skill, iterate on the description or the keyword patterns until they do. Doing this *before* the PR means CI's smoke test isn't the first time your routing gets exercised.

Requires a **debug build** of the app installed (`run-as` doesn't work on release builds) and `adb` on your PATH. See `./tools/sideload-android --help` for flags — alternate package name, device serial, skip-rebuild, skip-validate, skip-restart. Useful `adb logcat` filters while iterating:

```bash
# Your skill's own log output (WASM skills, via ari::log)
adb logcat -s AriSkill

# Engine-level events — skill loading, errors, startup counts
adb logcat -s EngineModule AriEngine SkillUpdateWorker AssetResolver
```

### 7. Open a pull request

```bash
git checkout -b coin-flip
git add skills/coin-flip
git commit -m "Add coin-flip skill"
git push -u origin coin-flip
```

Then open a PR against `ari-digital-assistant/ari-skills`. CI runs the same `validate` check plus a smoke test in a headless engine fixture. If it passes and a maintainer approves it, the merge bot signs the bundle and publishes it. Within minutes, every Ari user can install your skill from Settings → Skills → Browse.

### 8. Iterating

Found a bug, want to add a new keyword? Bump `metadata.ari.version`, open another PR. Once merged, every installed copy auto-updates on next app launch (within the semver range of the user's installed engine).

## Declarative response options

| Field | What it does |
|---|---|
| `response` | Fixed string. Use for deterministic replies. |
| `response_pick` | List of strings. One picked at random per invocation. |
| `response_template` | Mustache-style template with `{{var}}` placeholders. Use with `action` when the frontend or engine fills in values. |
| `action` | An action envelope the frontend renders (cards, alerts, notifications, launch_app, search, open_url, clipboard, dismissals). Combine with `response_template` to give the user verbal feedback. Full vocabulary: [action-responses.md](action-responses.md). |

Example combining a template and an action:

```yaml
    declarative:
      response_template: "Opening {{target}}."
      action:
        v: 1
        launch_app: "{{target}}"
```

The capture mechanism for `{{target}}` from the input is documented in [skill-system.md](skill-system.md).

## Beyond text responses — cards, alerts, notifications

Most skills can return a plain string and call it done ("Heads.", "It's quarter past four."). But the response surface is much richer than that: skills can also emit a structured **action envelope** that asks the frontend to render UI primitives, fire alerts, copy to the clipboard, launch apps, and more.

A timer skill, for example, emits one envelope per "set a timer" utterance carrying:

- a **card** with a live countdown rendered inline in the chat — icon + title, big monospace clock, blinking colon, accent-coloured progress bar that goes red as the deadline approaches;
- an `on_complete.alert` attached to the card — fires automatically at the deadline as a critical, full-takeover alert with looping audio + Siri-style speech, a dedicated lock-screen takeover surface, and a Stop action button that doesn't require unlocking the device;
- a paired ongoing **notification** with an OS-rendered chronometer countdown for the shade.

You declare what you want; the frontend renders it. None of that UI lives in the skill — your skill just emits primitives that say "card with these fields", "alert with this urgency", "notification with this importance". On Android the GenericCard composable renders countdowns and progress bars, AlertService runs the audio loop, AlertActivity is the alarm-clock takeover. On future Linux it'll be GTK + libnotify + GStreamer. Your skill doesn't change.

The same envelope vocabulary covers `launch_app`, `search`, `open_url`, `clipboard`, dismissals, and asset-bundled icons/sounds. Anything visual or audible you'd want a skill to do, you do by composing primitives — there's no "kind" enum to add to, no platform-specific renderer to publish.

For declarative skills, you can put a static envelope under `declarative.action`. For WASM skills, build it dynamically with the typed `presentation` builder in the Rust SDK.

```yaml
# Declarative example — single-shot launch
declarative:
  response_template: "Opening {{target}}."
  action:
    v: 1
    launch_app: "{{target}}"
```

```rust
// WASM example — minimal envelope
use ari_skill_sdk::presentation as p;
let json = p::Envelope::new()
    .speak("Copied that to your clipboard.")
    .clipboard("the text to copy")
    .to_json();
ari::respond_action(&json)
```

Full reference (every field, every primitive, every reserved id, asset bundling rules, lock-screen takeover semantics): **[action-responses.md](action-responses.md)**.

## Server-backed settings

Some settings can't be a plain text box. If your skill talks to a user's own server, the useful thing to ask for isn't an opaque entity id pasted by hand — it's a dropdown of the *actual* options that server reports, or a tick that confirms the URL and token you've been given actually work. That's what this is: a WASM skill can drive its own settings UI at settings-time, fetching live data over the network to populate a dropdown or validate a credential, instead of making the user copy ids out of a web console and pray.

There are two pieces, and they compose.

### `dynamic_select` — a dropdown the skill fills in

A normal `select` field declares its `options:` statically in the manifest. A `dynamic_select` field declares **no** `options:` — your skill fetches them at settings-time and hands them back. The user sees a dropdown; the persisted value is the chosen option's `value` (the `label` is display-only). From `execute()` you read it back with `ari::setting_get("<key>")` exactly like any other setting.

### `validate: true` — an inline ✓/✗ on any field

Add `validate: true` to *any* field (a `secret` token, a `text` URL, anything) and the UI shows an inline check result next to it: a green ✓ with a short confirmation message, or a red ✗ with an error. Your skill decides which by checking the value and answering. Use it on the credential field so the user knows their token works *before* they leave the settings screen, not the first time a voice command fails.

### `depends_on: [..]` — what the query needs, and when it fires

Both behaviours need to know which sibling fields to read. Declare them with `depends_on: [<key>, ...]` — the keys of the other fields in the same settings form your query needs (the server URL and the token, typically). The host auto-fires the query (debounced) once **all** the listed dependencies have a non-empty committed value, and re-fires it whenever any of them changes. Empty or absent dependencies → it never auto-fires; the field just shows a "fill the other fields first" placeholder. A field with no `depends_on` is never auto-queried.

### The committed-values caveat (read this twice)

The query sees its dependencies' **committed** values, not live keystrokes. On Android a `text`/`secret` field commits on focus-loss — when the user tabs away or taps another field. So the dropdown populates and the ✓/✗ appears once the user moves *off* the URL and token fields, not character-by-character as they type. This is deliberate v1 behaviour: it's one fetch when the inputs settle, not one per keystroke. Tell users (in your skill's setup prose) to fill the fields and move on; the rest fills itself in.

### The `settings_query` export

To drive any of this, export one more function alongside `score`/`execute`:

```text
settings_query(ptr: i32, len: i32) -> i64
```

Same ABI as `execute` — read the input with `ari::input(ptr, len)`, return with `ari::respond_action(&json)`.

**Input** is JSON identifying which field is being queried and the committed values of that field's `depends_on` siblings:

```json
{ "field": "agent_id", "values": { "base_url": "http://homeassistant.local:8123", "token": "ey…" } }
```

**Output** is JSON the host decodes:

```json
{ "ok": true, "options": [ { "value": "conversation.x", "label": "OpenAI Agent" } ] }
```

| Key | When |
|---|---|
| `ok` | Always. `true` if the query succeeded, `false` if it failed |
| `error` | On failure — a user-facing message rendered next to the ✗ |
| `options` | For a `dynamic_select` — the dropdown options (`{value, label}` pairs) |
| `message` | For a `validate` success — a short confirmation rendered next to the ✓ ("Connected") |

No new host imports are involved. The query runs inside your normal WASM sandbox and reuses what you already have: `ari::http_request` (so the skill must declare the `http` capability) for the fetch, and `ari::t(...)` for localized option labels and messages.

### What you get for free

Declare the fields and write the export — the frontend gives you the whole interaction:

- **debounced auto-fetch** when the dependencies are committed and non-empty, re-fetching when they change;
- a **Checking…** spinner while `settings_query` runs;
- the **populated dropdown** (`dynamic_select`) or the **✓ Connected** / **✗ error** result (`validate`);
- a **Retry** button on failure;
- a **"fill the fields first" placeholder** when the dependencies are still empty.

None of that UI lives in your skill. You answer `{field, values}` with `{ok, options/message/error}`; the frontend renders the rest.

### Worked example: the Home Assistant agent dropdown

The Home Assistant skill lets the user pick which HA *conversation agent* to route commands to. The list of agents lives on their server, so it's a `dynamic_select`. The token field validates itself at the same time.

The manifest declares the fields (`skills/home-assistant/SKILL.en.md`):

```yaml
    settings:
      - key: base_url
        label: "Home Assistant URL"
        type: text
        required: true
      - key: token
        label: "Long-lived access token"
        type: secret
        required: true
        validate: true
        depends_on: [base_url, token]
      - key: agent_id
        label: "Conversation agent entity (blank = HA default/local)"
        type: dynamic_select
        required: false
        depends_on: [base_url, token]
```

Both the `token` (validated) and the `agent_id` (dynamic dropdown) depend on `base_url` and `token` — so as soon as the user has entered a URL and a token and moved off them, the host fires `settings_query` for each. One round-trip checks the credential and shows ✓; another fetches the agent list and fills the dropdown.

The export parses the input, requires both `base_url` and `token`, hits HA's `/api/states` endpoint once, maps transport/auth failures to a friendly error, and then branches on `field`: for `agent_id` it returns the parsed agents as options; for anything else (the validated token field) it returns a "Connected" message. This is the real structure from `skills/home-assistant/src/lib.rs`:

```rust
#[no_mangle]
pub extern "C" fn settings_query(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    let result = handle_settings_query(input);
    ari::respond_action(&result)
}

fn handle_settings_query(input: &str) -> String {
    use ari::settings::{parse_query_input, SelectOpt, SettingsResult};

    let q = match parse_query_input(input) {
        Some(q) => q,
        None => return SettingsResult::error("bad query input").to_json(),
    };

    // Both deps are required before we can talk to the server.
    let base_url = match q.value("base_url").filter(|s| !s.trim().is_empty()) {
        Some(s) => s,
        None => return SettingsResult::error("Home Assistant isn't set up yet.").to_json(),
    };
    let token = match q.value("token").filter(|s| !s.trim().is_empty()) {
        Some(s) => s,
        None => return SettingsResult::error("Home Assistant isn't set up yet.").to_json(),
    };

    // One GET /api/states — reuses the same http_request import execute() uses.
    let resp = ari::http_request("GET", &states_url(base_url), &[("Authorization", &bearer(token))], None);
    if let Some(err) = http_error(resp.status) {
        return SettingsResult::error(err).to_json();
    }

    match q.field.as_str() {
        // dynamic_select → return the dropdown options
        "agent_id" => {
            let opts: Vec<SelectOpt> = parse_conversation_agents(resp.body.as_deref().unwrap_or(""))
                .into_iter()
                .map(|(value, label)| SelectOpt { value, label })
                .collect();
            SettingsResult::options(opts).to_json()
        }
        // validate: true → a green-tick confirmation message
        _ => SettingsResult::validated("Connected to Home Assistant.").to_json(),
    }
}
```

The `ari::settings` helpers do the JSON for you: `parse_query_input(&str) -> Option<SettingsQueryInput>` gives you `.field` and `.value("<dep_key>") -> Option<&str>`; `SettingsResult::{options(Vec<SelectOpt>), validated(&str), error(&str)}` builds the reply and `.to_json()` serialises it. They're pure (no WASM ABI), so you can unit-test your query logic natively. They live behind the SDK's `settings` Cargo feature — enable it on the dependency:

```toml
[dependencies]
ari-skill-sdk = { path = "../../sdk/rust", features = ["http", "settings"] }
```

(The real skill localizes its strings through `ari::t(...)` and maps a richer set of HTTP errors — see `skills/home-assistant/src/lib.rs` for the full thing. The skeleton above is trimmed to the shape, not the polish.)

## Action buttons and OAuth sign-in

Sometimes a setting isn't a *value* the user types — it's a *thing that has to
happen*. "Sign in." "Reconnect." "Authorise access." For that there's a third
settings field type, `type: action`: a button on the settings screen that, when
pressed, calls a `settings_action` WASM export. The headline use is **OAuth** —
the user taps "Sign in", a browser opens, they approve, and your skill comes
back holding a token. No copy-pasting long-lived tokens out of a web console.

### The `authorize` capability

`ari::authorize` does the browser round-trip for you: it opens an authorization
URL in the system browser, waits for the redirect back to your `redirect_uri`,
and returns the callback query parameters. **Your skill builds the URL and owns
the OAuth protocol** (the `state` check, PKCE, swapping the `code` for a token);
the host just drives the browser.

Declare the capability and enable the SDK features:

```yaml
    capabilities: [http, authorize, storage_kv]
```

```toml
[dependencies]
# `authorize` for the browser round-trip, `crypto` for PKCE, `http` for the
# token exchange, `settings` for the action-input parser.
ari-skill-sdk = { path = "../../sdk/rust", features = ["http", "authorize", "crypto", "settings"] }
```

> **You don't host anything and you don't need to own a domain.** Call
> `ari::oauth_redirect_uri()` for the redirect URI and register *that value*
> with your OAuth provider. For **standard OAuth2** (Google, GitHub, …) use the
> `client_id` your provider issued you. For **IndieAuth** (Home Assistant,
> Mastodon, …) use the shared Ari client id `https://heyari.dev/oauth/client`.
> The host opens the browser, binds a nonce to the flow, and hands your skill
> the callback params.

### Declare the button

An `action` field renders a button. `depends_on` greys it out until its
dependencies are filled (no point letting someone sign in before they've given
you the server URL):

```yaml
    settings:
      - key: base_url
        label: "Home Assistant URL"
        type: text
        required: true
      - key: sign_in
        label: "Sign in with Home Assistant"
        type: action
        depends_on: [base_url]
```

### Handle the press

Export `settings_action` alongside `score`/`execute`. The host calls it with
`{ "action": "<key>", "values": { ...committed settings... } }` and renders your
`{ok, error?, message?}` reply (green ✓ message or red ✗ error) — same shape as
`settings_query`. Use `ari::settings::parse_action_input` to read it.

```rust
#[no_mangle]
pub extern "C" fn settings_action(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    ari::respond_action(&handle_sign_in(input))
}

fn handle_sign_in(input: &str) -> String {
    use ari::settings::{parse_action_input, SettingsResult};
    let a = parse_action_input(input).unwrap();
    let base_url = a.value("base_url").unwrap_or("");

    // 1. PKCE + state, both from the ungated CSPRNG (ari::rand_u64).
    let verifier = make_verifier();                 // 32 random bytes -> base64url
    let challenge = ari::crypto::base64url_nopad(&ari::crypto::sha256(verifier.as_bytes()));
    let state = make_state();

    // 2. Ask the host for its redirect URI, build the authorize URL, open the
    //    browser. For IndieAuth the client_id is the shared Ari client page;
    //    for standard OAuth2 it's the client_id your provider issued you.
    let redirect = ari::oauth_redirect_uri();
    let auth_url = build_authorize_url(
        base_url, "https://heyari.dev/oauth/client", &redirect, &state, &challenge,
    );
    let res = ari::authorize(&auth_url, &redirect, 300_000);  // 5-minute timeout
    if !res.ok {
        let msg = match res.error.as_deref() {
            Some("no_browser") => "I couldn't open a browser to sign in.",
            _ => "Sign-in didn't complete. Please try again.",
        };
        return SettingsResult::error(msg).to_json();
    }

    // 3. Verify state, then swap the code for a token over HTTP.
    if res.get("state") != Some(state.as_str()) || res.get("error").is_some() {
        return SettingsResult::error("Sign-in couldn't be verified.").to_json();
    }
    let code = res.get("code").unwrap_or("");
    let body = build_exchange_body(code, &verifier, &redirect);
    let resp = ari::http_request("POST", TOKEN_ENDPOINT,
        &[("Content-Type", "application/x-www-form-urlencoded")], Some(&body));
    // ...parse the refresh token out of resp.body...

    // 4. Persist it. Store the *refresh token*, never the auth code.
    ari::setting_set("token", &refresh_token);
    SettingsResult::validated("Signed in.").with_refresh().to_json()
}
```

A few things the real skill (`skills/home-assistant/src/lib.rs`) does that are
worth copying:

- **`res.error` values:** `"cancelled"`, `"timeout"`, `"no_browser"`,
  `"mismatch"` (the callback didn't match `redirect_uri`), `"bad_request"`,
  `"bad_response"`. Map them to friendly messages.
- **Always verify `state`** against the value you generated, and reject if the
  callback carries an `error` param — that's your CSRF defence.
- **Store the refresh token, not the authorization code.** The code is
  single-use and short-lived; the refresh token is what you re-exchange later.
- **`.with_refresh()`** on the result tells the settings screen to reload after
  a successful sign-in, so other fields (like a now-unlocked dropdown) re-query.

The full ABI for `ari::authorize` and the `authorize` capability is in
[wasm-sdk.md](wasm-sdk.md#oauth-sign-in-features--authorize).

## When you actually need WASM

Reach for WASM only if your skill needs to:

- Make HTTP calls (weather, news, public APIs)
- Do non-trivial parsing of the user input
- Maintain state across invocations (`storage_kv`)
- Compute something the declarative templates can't express

A WASM skill declares its capabilities and ships a `skill.wasm` module alongside `SKILL.en.md`. The host surface is deliberately small: logging, time/entropy, capability checks, the skill's own settings (`setting_get`/`setting_set`) and i18n helpers come for free; `http_fetch`/`http_request`, `storage_get`/`storage_set`, and `authorize` (OAuth sign-in) are capability-gated. The full ABI — every import, every signature, the SDK feature flags — is in [wasm-sdk.md](wasm-sdk.md#host-imports-all-in-the-ari-module). If you need something that isn't there, file an issue first.

Two SDKs are available:

- **Rust** — `sdk/rust/` and template at `templates/rust/`
- **AssemblyScript** — `sdk/assemblyscript/` and template at `templates/assemblyscript/`

Copy a template, edit `SKILL.en.md` + the source file, run `build.sh`, and you're done. Full docs: [wasm-sdk.md](wasm-sdk.md).

## How your skill gets matched (the two layers)

When a user speaks, Ari does this:

1. **Keyword matcher** runs first — fast, deterministic, free. It scores
   every installed skill against the user's input using the `matching.patterns`
   you declared. If something clears the threshold, that skill executes. Done.

2. **If nothing matched**, Ari optionally consults the **FunctionGemma router** —
   a small (~250MB) on-device language model fine-tuned for routing. It sees
   your skill's `name` and `description`, and the user's input, and picks one.
   This is the safety net for paraphrases the keyword matcher missed.

So a user saying *"flip a coin"* always lands on coin-flip via keywords. A user
saying *"let's leave it to chance, heads or tails"* won't trigger any keyword
patterns — but the FunctionGemma router can still route them to coin-flip,
**because it understood the description**.

This is why your `description` matters more than you might think. The keyword
matcher only ever reads `matching.patterns`. The FunctionGemma router reads
the `description`. Two completely different consumers, both important.

### Writing a description that works for the router

The router is a model. It pattern-matches on semantic similarity. Two rules:

1. **Lead with what the skill does in plain English.** "Tells the current
   time", "Flips a coin", "Sends an email". The router pulls the skill's
   purpose from the first sentence.

2. **Enrich with semantic keywords for the second sentence.** Don't just say
   "Use when the user asks the time" — say "Use when the user asks what
   time it is, what hour it is, whether it is morning or afternoon, or
   anything about the current time of day." The router picks up phrases
   like "morning or afternoon" and learns to route them.

A weak description means the router won't catch paraphrases. A rich one
gives you free coverage for utterances you never thought of.

### Example utterances

Every skill must include example utterances in `metadata.ari.examples`.
These feed directly into the FunctionGemma router's training dataset.
The validator enforces a minimum of 5, but aim for 15-30 for good
coverage.

Each entry has a `text` field (the user utterance) and an optional `args`
field (the JSON arguments the function call should produce). Parameterless
skills omit `args`:

```yaml
    examples:
      - text: "flip a coin"
      - text: "toss a coin please"
      - text: "heads or tails"
      - text: "let's leave it to chance"
      - text: "can you flip a coin for me"
```

Parameterised skills include `args`:

```yaml
    examples:
      - text: "open spotify"
        args:
          app_name: Spotify
      - text: "launch the camera"
        args:
          app_name: Camera
      - text: "fire up the music player"
        args:
          app_name: Music Player
```

Cover paraphrases, indirect language, and conversational filler ("can you",
"please", "I need"). The point is to teach the router that all the natural
ways a user might phrase a request should land on your skill, not just the
rigid ones your keyword patterns catch.

## Rules of the road

These will save your PR from review friction:

1. **Be honest in the description.** "Flips a coin" is fine. "AI-powered randomness engine for decision-making" is not.
2. **Source language only.** Don't auto-translate strings. Declare `languages: [en]` (or whatever you actually wrote it in) and stop. Translations belong in a translation platform, not in your PR. When you're ready to add a second language, see **[i18n.md](i18n.md)** — Ari supports per-locale `SKILL.{locale}.md` files plus a `strings/{locale}.json` translation table that the SDK's `t()` / `format_*` helpers read from.
3. **Don't squat namespaces.** Pick a `metadata.ari.id` reverse-DNS prefix you actually control or have a clear claim to.
4. **Tight keywords beat clever regex.** A short keyword list with `specificity: high` will outperform an over-eager regex that fires on every other utterance.
5. **Declare exactly the capabilities you need.** Asking for `http` when you don't use it will get flagged in review.
6. **Don't reinvent built-ins.** If Ari already ships a `CalculatorSkill`, don't publish a competing one — improve the built-in via a PR to the main Ari repo instead.
7. **No third-party-specific lock-in when a generic API exists.** "Open my podcast app" should use `launch_app` with a generic target, not hard-code one app's package name.

## Where to ask questions

- Issues: <https://github.com/ari-digital-assistant/ari-skills/issues>
- Main project: <https://github.com/ari-digital-assistant/ari>
- Spec for the underlying AgentSkills format: <https://agentskills.io/specification.md>
