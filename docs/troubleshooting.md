# Troubleshooting

Every trap we know about, and how to get out of it.

---

## My pattern never matches

The single most common problem, and it's almost always one of these five.

### 1. Your keyword has an apostrophe or a capital letter

Patterns are matched against **normalised** text, never raw speech. By the
time matching happens the input has been lowercased, had contractions
expanded, and had punctuation replaced with spaces.

```yaml
- keywords: [what's, time]   # can NEVER match — it's "what is" by then
- keywords: [Spotify]        # can NEVER match — it's lowercase by then
```

The expansions: `what's`/`whats`→`what is`, `it's`→`it is`, `i'm`→`i am`,
`don't`→`do not`, `doesn't`→`does not`, `can't`→`cannot`, `won't`→`will not`,
`isn't`→`is not`, `aren't`→`are not`, `didn't`→`did not`, `there's`→`there is`,
`here's`→`here is`, `that's`→`that is`, `let's`→`let us`, `we're`→`we are`.

Italian elisions are stripped: `l'ora` → `l ora`. Match `ora`.

### 2. All keywords must be present

A keyword pattern matches only when **every** word in the list is in the
utterance. `[roll, dice]` needs both. If you want either, write two patterns.

### 3. Matching is whole-word — there's no stemming

`[toss]` does not match "tossed". `[roll]` does not match "rolling". List the
forms you want, or use a `regex` pattern.

### 4. Something else won first

Run with `--debug` and look at the trace:

```bash
cargo run -p ari-cli -- --debug --extra-skill-dir path/to/skill "your input"
```

```
[ari]   current_time (High): 0.000
[ari]   com.example.diceroll (High): 0.950
[ari] winner: com.example.diceroll (round 1)
```

If another skill scored higher, or won an earlier round, that's your answer.
Rounds go by specificity: a `high`-specificity skill at 0.85 wins in round 1
before any `medium` skill is even considered.

### 5. Your score didn't clear the threshold

| Round | `high` | `medium` | `low` |
|---|---|---|---|
| 1 | 0.85 | — | — |
| 2 | 0.75 | 0.85 | — |
| 3 | 0.60 | 0.70 | 0.80 |

Your skill's score is the **maximum** weight among matching patterns — they
don't add up. A `medium` skill with `weight: 0.7` never wins anything. Either
raise the weight or set `specificity: high`.

## My skill fires when it shouldn't

Usually keywords that are too generic, or `specificity: low` on something that
should be narrow.

Test the negatives explicitly. A skill matching `[open]` will hijack "open the
window", "open a bank account" and "open question". Add the second keyword
that makes it unambiguous.

## The router never picks my skill

The router only sees utterances the **keyword matcher missed**. If your own
patterns already win an utterance, the router is never consulted for it.

Beyond that, the router reads exactly one field: `description`. Not your
patterns, not your body text. If it's vague, rewrite it — first sentence what
the skill does, second sentence when to use it, packed with the words a real
person would say.

Then sideload the skill and try the phrasings from your `examples` as actual
utterances. That's the only honest test.

## CI failed with "router-example poaching"

Another skill's keywords win one of your examples. Re-word the example so no
keyword set catches it. Full explanation:
[publishing.md](publishing.md#the-no-poaching-gate).

## Install fails with "couldn't reach the registry"

**Check your bundle size first.** Bundles are capped at 8 MiB and going over
produces exactly this misleading message — it has nothing to do with your
network.

Usually it's `assets/`. Compress images, use WebP, and don't ship audio you
could reference with a `system.*` sound token.

## Capability and feature mismatches

Two halves that have to agree, failing in two different ways.

| Symptom | Cause | Fix |
|---|---|---|
| Compile error, function not found | SDK feature missing | Add it to `features = [...]` |
| Install rejected, "missing capabilities" | Manifest capability missing | Add it to `capabilities: [...]` |
| Install rejected, module imports X | You imported a gated function you didn't declare | Declare it |
| Works on Android, rejected on CLI | The CLI grants only the pure-frontend set | `--host-capabilities http,storage_kv,…` |

Declaring a capability whose functions you never import is *not* an error —
LTO strips the unused import, so the sneak guard sees nothing. It'll still get
flagged in review.

## My WASM skill returns "(skill error)"

The host says this when `execute` violated the contract. Check:

- **Fuel exhaustion.** 50,000,000 units per call, tens of milliseconds. An
  accidental unbounded loop, or a big JSON parse, will hit it.
- **Memory.** Default 16 MiB, range 1–16. If you set `memory_limit_mb: 1`, a
  `std` Rust skill won't even start — it needs ~1.1 MiB just to boot.
- **A bad response tag.** Only `0x00` (text) and `0x01` (action) are legal.
  Use `respond_text` / `respond_action` and don't hand-roll the packing.
- **A panic.** Add `ari::log` calls and watch `adb logcat -s AriSkill`.

## State disappears between calls

By design — every invocation gets a fresh store. Nothing in a `static`, a
`lazy_static` or a global survives.

Use `storage_kv` for anything that must persist. Note it needs the capability
*and* the `storage` feature, and it's a different thing from
`setting_get`/`setting_set` (which read your settings and need no capability).

## My card stacks up instead of updating

Your card `id` isn't stable. Re-emitting a card with an existing id replaces
it; a fresh random id every time adds a new one. Pick an id derived from what
the card represents, not from the clock.

## My alert doesn't take over the screen

`full_takeover` is ignored unless `urgency` is `critical` — that's a
deliberate safety gate. You also need `capabilities: [critical_alert]`, and on
Android the user must have granted `USE_FULL_SCREEN_INTENT`; the frontend
prompts them when a skill declaring it is installed.

## My settings dropdown never populates

Three things to check.

**The dependencies aren't committed yet.** `settings_query` sees each
`depends_on` field's *committed* value, not live keystrokes. On Android a text
field commits on focus loss — so the query fires when the user moves *off* the
field, not as they type. Say so in your setup text.

**A dependency is empty.** The host only auto-fires once **all** listed
dependencies are non-empty. Until then the field shows a placeholder.

**No `depends_on` at all.** A field without it is never auto-queried.

## `show_when` made my field vanish forever

`show_when.key` must name a real sibling field in the same `settings` list.
A typo is a parse error, not a silent hide — the validator will tell you which
field is wrong.

## `--extra-skill-dir` loads nothing

You're pointing at a directory with no `SKILL.en.md` (or legacy `SKILL.md`) at
its top level. When the CLI doesn't find one it treats the path as a registry
*root* and walks the children instead — which is why pointing at a build
directory produces a confusing "no SKILL.md found" for `target/`.

Point it at the skill directory itself, or at the parent of several skill
directories. Nothing in between.

## AssemblyScript: the host can't write my input

You forgot to re-export the allocator:

```typescript
export { ari_alloc };
```

The skill compiles and loads, and then receives nothing.

## AssemblyScript: module fails to instantiate

You're importing `env::abort`, which the host doesn't provide. Compile with
`--use abort=`. The bundled `build.sh` already does.

## The validator can't find its binary

```
could not find the ari-skill-validate binary
```

Pick one:

- Clone `ari-engine` next to `ari-skills` (easiest)
- `cargo install --git https://github.com/ari-digital-assistant/ari-engine ari-skill-validate`
- `export ARI_SKILL_VALIDATE=/path/to/binary`

CI runs it regardless, so this only blocks local iteration.

## Sideloading does nothing

- The app must be a **debug** build — `run-as` is refused on release builds.
- `adb` must be on your `$PATH` and the device authorised (`adb devices`).
- The tool force-stops and relaunches the app so the engine rescans. If you
  passed `--skip-restart`, restart it yourself.

## Still stuck

Open an issue with the manifest, the utterance, and the `--debug` trace:
<https://github.com/ari-digital-assistant/ari-skills/issues>
