# Manifest reference

Every field of `SKILL.en.md`, what it does, and whether you need it.

A manifest is an [AgentSkills](https://agentskills.io/specification.md)
document: YAML frontmatter, then a markdown body. Standard fields sit at the
top level; everything Ari-specific lives under `metadata.ari.*`.

Localised variants are `SKILL.{locale}.md` — see [i18n.md](i18n.md). A bare
`SKILL.md` is still accepted and treated as English, but a directory may not
contain both it and `SKILL.en.md`.

---

## Top level

| Field | Required | Notes |
|---|---|---|
| `name` | **Yes** | 1–64 chars, `[a-z0-9-]` only. No leading, trailing or doubled hyphens. **Must equal the directory name.** |
| `description` | **Yes** | 1–1024 chars. Read by the router — see [writing one](#writing-a-description-the-router-can-use). |
| `license` | No | SPDX identifier or free text. |
| `compatibility` | No | Free-text environment notes. |
| `metadata` | No in AgentSkills, **yes in practice** | Without `metadata.ari` your file is a valid AgentSkills doc but not an Ari skill, and the validator says so. |

The markdown body below the frontmatter is yours. It's shown to developers and
reserved as input for future routing models. Keep it short and factual.

## `metadata.ari`

| Field | Required | Default | Notes |
|---|---|---|---|
| `id` | **Yes** | — | Reverse-DNS, unique across the registry. Keys storage, signing and updates. Never change it. |
| `version` | **Yes** | — | Semver. Bump on every submission. |
| `engine` | **Yes** | — | Semver range against the engine, e.g. `">=0.3,<0.4"`. |
| `author` | No | — | Free text. Include it anyway. |
| `homepage` | No | — | URL. |
| `capabilities` | No | `[]` | See [reference-capabilities.md](reference-capabilities.md). |
| `platforms` | No | all | Allowlist, e.g. `[android]`. An escape hatch — capabilities are the real contract. |
| `languages` | No | `[]` | Source languages you actually wrote. Never machine-translated. |
| `type` | No | `skill` | `skill` or `assistant`. See [assistant-skills.md](assistant-skills.md). |
| `specificity` | No | `medium` | `low` / `medium` / `high`. Feeds the ranking rounds. |
| `matching` | **Yes** for `type: skill` | — | Keyword and regex patterns. |
| `declarative` | XOR `wasm` | — | Response config for a code-free skill. |
| `wasm` | XOR `declarative` | — | Module config for a WASM skill. |
| `assistant` | **Yes** for `type: assistant` | — | See [assistant-skills.md](assistant-skills.md). |
| `examples` | No to parse, **yes to publish** | `[]` | Router training utterances. Under 5 raises a validator warning. |
| `settings` | No | `[]` | The skill's settings screen. |
| `fallback` | No | — | `{requires_setting: <key>}` — marks the skill as a fallback tier that's only eligible once that setting has a value. |

Several of these are commonly believed to be required and aren't:
`capabilities`, `languages`, `specificity` and `author` all have defaults. Set
them anyway — an empty `capabilities: []` says "I checked", an absent one says
"I didn't think about it".

### Writing a `description` the router can use

Two sentences. The first says what the skill does. The second says when to
use it, in the words a real person would use.

```yaml
description: >
  Flips a virtual coin and returns heads or tails. Use when the user asks to
  flip a coin, toss a coin, call it, or make a random either-or choice.
```

The keyword matcher never reads this field. The **router** does — it's a
model, matching on meaning. "call it" and "either-or choice" in that second
sentence buy you coverage for phrasings you'd never enumerate as keywords.

Vague description, unused skill. This is the highest-leverage field in the
file.

## `matching`

```yaml
    matching:
      patterns:
        - keywords: [flip, coin]
          weight: 0.95
        - regex: "^(heads|tails)$"
          weight: 0.9
      custom_score: false
```

| Field | Required | Default | Notes |
|---|---|---|---|
| `patterns` | **Yes** | — | At least one entry. |
| `patterns[].keywords` | XOR `regex` | — | List of words. **All** must be present, as whole words. |
| `patterns[].regex` | XOR `keywords` | — | Rust `regex` syntax. Matched with `is_match`. |
| `patterns[].weight` | No | `1.0` | The score this pattern contributes. |
| `custom_score` | No | `false` | WASM only. Calls your `score` export instead. Rarely worth it. |

**Semantics:**

- A keyword pattern matches only if **every** word is present, as a **whole
  word**. `[toss, coin]` does not match "tossed a coin".
- Your skill's score is the **maximum** weight across matching patterns. They
  don't accumulate.
- Keywords are lowercased when the manifest is parsed.
- Everything is matched against **normalised** input — see
  [normalisation](#input-normalisation) below. Never put an apostrophe or a
  capital letter in a pattern.

### How the score is used

Three ranking rounds, each with a threshold per specificity level. The first
skill to clear its round's bar wins.

| Round | `high` | `medium` | `low` |
|---|---|---|---|
| 1 | 0.85 | — | — |
| 2 | 0.75 | 0.85 | — |
| 3 | 0.60 | 0.70 | 0.80 |

A confident `high` skill therefore wins before a broad `low` catch-all gets
considered at all. If no round produces a winner, the input goes to the
router.

### Input normalisation

Patterns — and the input your `execute` receives — are the **output** of the
engine's normaliser, not raw speech. In order:

1. Lowercased.
2. English contractions expanded: `what's`→`what is`, `it's`→`it is`,
   `i'm`→`i am`, `don't`→`do not`, `doesn't`→`does not`, `can't`→`cannot`,
   `won't`→`will not`, `isn't`→`is not`, `aren't`→`are not`,
   `didn't`→`did not`, `there's`→`there is`, `here's`→`here is`,
   `that's`→`that is`, `let's`→`let us`, `we're`→`we are`.
   Italian elisions are stripped (`l'ora` → `l ora`).
3. Every character that isn't alphanumeric, whitespace, or one of
   `+ - * / . % ^` becomes a space. A comma survives only between two digits.
4. Runs of whitespace collapse to one space.
5. English number words become digits.

So `"What's 2 o'clock?"` reaches your skill as `"what is 2 o clock"`.

## `declarative`

```yaml
    declarative:
      response_pick: ["Heads.", "Tails."]
```

Exactly one response field, plus an optional action:

| Field | Notes |
|---|---|
| `response` | A fixed string. |
| `response_pick` | A list; one chosen at random per invocation. Must not be empty. |
| `response_template` | A string with `{{placeholder}}` slots. |
| `action` | An [action envelope](reference-actions.md). Combines with any of the above. |

Response strings that match a key in `strings/<locale>.json` are resolved
through it; anything else is emitted verbatim. That's how a declarative skill
becomes translatable without changing shape — see [i18n.md](i18n.md).

```yaml
    declarative:
      response_template: "Opening {{target}}."
      action:
        v: 1
        launch_app: "{{target}}"
```

## `wasm`

```yaml
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
```

| Field | Required | Default | Notes |
|---|---|---|---|
| `module` | **Yes** | — | Filename relative to the skill directory. |
| `memory_limit_mb` | No | **16** | Must be 1–16. A `std` Rust skill needs ~1.1 MiB to start. |

Fuel is capped at 50,000,000 units per call — tens of milliseconds of compute.
Each call gets a fresh store, so nothing survives between invocations. Use
`storage_kv` for state.

## `examples`

```yaml
    examples:
      - text: "settle this for me, heads or tails"
      - text: "open spotify"
        args:
          app_name: Spotify
```

| Field | Required | Notes |
|---|---|---|
| `text` | **Yes** | The user utterance. |
| `args` | No | The arguments the router should produce. Read them back with `ari::args()`. |

These are **router training data**. The router only ever sees utterances the
keyword matcher missed, so a good example is one your own patterns do **not**
match. CI rejects examples that any skill's keywords already win — see
[the no-poaching gate](publishing.md#the-no-poaching-gate).

Five is the enforced minimum. Aim for 15–30, covering paraphrases, indirect
phrasing and conversational filler.

Assistant-type skills are exempt.

## `settings`

Each entry declares one field on the skill's settings screen.

```yaml
    settings:
      - key: base_url
        label: "Server URL"
        type: text
        required: true
        help_text: "e.g. http://homeassistant.local:8123"
```

| Property | Default | Notes |
|---|---|---|
| `key` | — | The identifier you pass to `ari::setting_get`. |
| `label` | — | Shown to the user. |
| `type` | — | See below. |
| `required` | `false` | |
| `default` | — | Pre-filled value. |
| `options` | `[]` | For `select`: `{value, label, download_url?, download_bytes?}`. |
| `show_when` | — | `{key, equals}` — show only when a sibling field has one of these values. `equals` accepts a string or a list. **The key must name a real sibling** or the manifest fails to parse. |
| `validate` | `false` | Show an inline ✓/✗ by asking the skill. WASM only. |
| `depends_on` | `[]` | Sibling keys whose values a query needs. |
| `help_text` | — | Explanatory line under the field. |
| `collapsed_group` | — | Puts the field in a collapsible section with this heading. |

Values are always persisted as strings. Parse them yourself.

### Settings fields

| `type` | Notes |
|---|---|
| `text` | Free text. |
| `secret` | Free text, masked, routed to encrypted storage. |
| `select` | Dropdown with static `options`. |
| `device_calendar` | Dropdown the **frontend** fills with the device's calendars. Declare no `options`. |
| `device_task_list` | Dropdown the **frontend** fills with the device's task lists. Declare no `options`. |
| `dynamic_select` | Dropdown **your skill** fills at settings time. Declare no `options`. Needs a `settings_query` export. WASM only. |
| `action` | A button that calls your `settings_action` export. WASM only. |

### `dynamic_select`

For settings that can't be typed from memory — "which of your server's agents
should I use?". Your skill fetches the options live.

Declare the field and what it depends on:

```yaml
      - key: agent_id
        label: "Conversation agent"
        type: dynamic_select
        required: false
        depends_on: [base_url, token]
```

Export `settings_query` alongside `score`/`execute`:

```rust
#[no_mangle]
pub extern "C" fn settings_query(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    ari::respond_action(&handle_query(input))
}

fn handle_query(input: &str) -> String {
    use ari::settings::{parse_query_input, SelectOpt, SettingsResult};

    let Some(q) = parse_query_input(input) else {
        return SettingsResult::error("bad query input").to_json();
    };
    let Some(base) = q.value("base_url").filter(|s| !s.trim().is_empty()) else {
        return SettingsResult::error("Fill in the server URL first.").to_json();
    };

    // … fetch over ari::http_request …

    SettingsResult::options(vec![SelectOpt {
        value: "conversation.x".into(),
        label: "OpenAI Agent".into(),
    }])
    .to_json()
}
```

The host calls you with `{"field": "agent_id", "values": {…committed
siblings…}}` and expects `{"ok": true, "options": [{value, label}]}` back.
`SettingsResult::{options, validated, error}` build the reply;
`.to_json()` serialises it. Needs `features = ["settings"]`.

You get the debounced fetch, the spinner, the populated dropdown, a Retry
button and a "fill the other fields first" placeholder for free.

**Read this bit twice:** the query sees each dependency's **committed** value,
not live keystrokes. On Android a text field commits on focus loss. So the
dropdown fills once the user moves *off* the URL and token fields, not as they
type. Say so in your setup text.

### `validate: true`

Same machinery, different outcome. Put it on any field — a token, a URL — and
your `settings_query` gets asked about it; return
`SettingsResult::validated("Connected")` for a green ✓ or
`SettingsResult::error("…")` for a red ✗. The user finds out their credentials
work before they leave the screen, rather than the first time a voice command
fails.

### OAuth sign-in

An `action` field renders a button. `depends_on` greys it out until its
dependencies are filled.

```yaml
      - key: sign_in
        label: "Sign in with Home Assistant"
        type: action
        depends_on: [base_url]
```

Export `settings_action`. The host calls it with `{"action": "<key>",
"values": {…}}` and renders your `{ok, error?, message?}` reply.

```rust
#[no_mangle]
pub extern "C" fn settings_action(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    ari::respond_action(&handle_sign_in(input))
}

fn handle_sign_in(input: &str) -> String {
    use ari::settings::{parse_action_input, SettingsResult};
    let a = parse_action_input(input).unwrap();

    // 1. PKCE + state from the ungated CSPRNG.
    let verifier = make_verifier();
    let challenge = ari::crypto::base64url_nopad(&ari::crypto::sha256(verifier.as_bytes()));
    let state = make_state();

    // 2. Ask the host for its redirect URI — never hardcode it.
    let redirect = ari::oauth_redirect_uri();
    let auth_url = build_authorize_url(&a, &redirect, &state, &challenge);
    let res = ari::authorize(&auth_url, &redirect, 300_000);
    if !res.ok {
        return SettingsResult::error(match res.error.as_deref() {
            Some("no_browser") => "I couldn't open a browser to sign in.",
            _ => "Sign-in didn't complete. Please try again.",
        })
        .to_json();
    }

    // 3. Verify state BEFORE trusting anything. This is your CSRF defence.
    if res.get("state") != Some(state.as_str()) || res.get("error").is_some() {
        return SettingsResult::error("Sign-in couldn't be verified.").to_json();
    }

    // 4. Exchange the code, then store the REFRESH token — never the code.
    let refresh_token = exchange(res.get("code").unwrap_or(""), &verifier, &redirect);
    ari::setting_set("token", &refresh_token);
    SettingsResult::validated("Signed in.").with_refresh().to_json()
}
```

Needs `capabilities: [http, authorize, storage_kv]` and
`features = ["http", "authorize", "crypto", "settings"]`.

- You don't need to own a domain. `ari::oauth_redirect_uri()` returns the
  host's own redirect; register that with your provider. For standard OAuth2
  use the `client_id` your provider issued. For IndieAuth (Home Assistant,
  Mastodon) use the shared Ari client id `https://heyari.dev/oauth/client`.
- `res.error` values: `"cancelled"`, `"timeout"`, `"no_browser"`,
  `"mismatch"`, `"bad_request"`, `"bad_response"`.
- `.with_refresh()` tells the settings screen to reload, so fields unlocked by
  the sign-in re-query.

A complete working implementation is in
[`skills/home-assistant/src/lib.rs`](../skills/home-assistant/src/lib.rs).

## Validation errors

The validator runs the engine's own loader, so anything it accepts, the engine
accepts. Common failures:

| Message | Fix |
|---|---|
| `` `name` must match the parent directory name `` | Rename one of them. |
| `` `name` may only contain lowercase letters, digits, and hyphens `` | No underscores, no capitals. |
| `` `metadata.ari.matching.patterns` must contain at least one entry `` | Add a pattern, or set `type: assistant`. |
| `exactly one of declarative/wasm` | You have both or neither. |
| `` `custom_score` requires a wasm skill `` | Declarative skills can't self-score. |
| `` `memory_limit_mb` must be 1..=16 `` | Clamp it. |
| `show_when` key not found | The key must name a sibling settings field. |
| `must contain at least 5 entries` | Warning, not an error — but write more examples. |

## See also

- [reference-capabilities.md](reference-capabilities.md) — what `capabilities` can contain
- [reference-actions.md](reference-actions.md) — what `action` can contain
- [reference-sdk.md](reference-sdk.md) — the functions behind `settings_query` and friends
- [i18n.md](i18n.md) — `SKILL.{locale}.md` and `strings/`
