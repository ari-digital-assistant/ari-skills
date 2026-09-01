# How the skill system works

You don't need this to write a skill. It's here for people who want to
understand the machinery, or change it.

## The trait

Every skill — built-in, declarative, WASM — is the same thing at the engine
boundary:

```rust
pub trait Skill: Send + Sync {
    fn id(&self) -> &str;
    fn specificity(&self) -> Specificity;
    fn score(&self, input: &str, ctx: &SkillContext) -> f32;
    fn execute(&self, input: &str, ctx: &SkillContext) -> Response;
}
```

Declarative and WASM skills are wrapped by adapters in `ari-skill-loader` that
implement this and delegate to the manifest or the module.

| Flavour | Lives in | Written by | Runtime |
|---|---|---|---|
| Built-in Rust | `ari-engine/crates/ari-skills` | Core team | Compiled in, always present |
| Declarative | this registry | Anyone | Manifest only |
| WASM | this registry | Anyone | Sandboxed wasmtime module |
| Assistant | this registry | Anyone | Not a `Skill` — a separate fallback path |

## Routing, in order

```
utterance
   ↓ normalize_input
   ↓
1. keyword scorer  ──── clears a threshold? ──→ execute, done
   ↓ nothing cleared
2. example phrases ──── a phrase matches? ──→ execute, done
   ↓ nothing matched
3. assistant ──── active? ──→ answer
   ↓ none
   "Sorry, I didn't understand that."
```

### 1. The keyword scorer

Deterministic, no model, no allocation per skill. At load time the engine
compiles a native `PatternScorer` from each manifest's `matching.patterns`.
**The WASM module is never called during scoring** — that's what lets the
registry scale to hundreds of skills without an FFI cost per utterance.

A keywords pattern matches when every word is present as a whole word; a regex
pattern matches on `is_match`. A skill's score is the maximum weight across
its matching patterns.

Three ranking rounds then apply thresholds by specificity:

| Round | `high` | `medium` | `low` |
|---|---|---|---|
| 1 | 0.85 | — | — |
| 2 | 0.75 | 0.85 | — |
| 3 | 0.60 | 0.70 | 0.80 |

First skill to clear its round's bar wins. A confident narrow skill therefore
beats a broad catch-all, regardless of raw score.

WASM skills can opt into `matching.custom_score: true`, which makes the loader
call the module's `score` export during ranking. It costs an instantiation per
skill per utterance, so it's discouraged.

### 2. Example phrases

Every skill declares example phrases in its manifest's `examples:` block, and
the engine matches them directly against anything the keyword scorer didn't
claim. No model is involved.

A phrase is a template: literal words plus `{slot}` placeholders that each bind
one or more words. `metti su {artist}` matches "metti su vasco rossi". The
match is anchored at both ends, so a phrase claims the whole utterance or
nothing — which is why you want several phrasings rather than one clever one.

Each phrase carries a `weight` (0..=1, same scale as `matching.patterns`)
saying how confidently that wording means your skill. Put explicit phrasings
high and anything another skill could plausibly claim low; the engine runs the
same ranking rounds it uses for keywords, so weight and `specificity` decide
who wins when two skills both match.

Phrase matching is a **fallback**: it only ever sees utterances the keyword
scorer didn't claim. That's the fact behind the
[no-poaching gate](publishing.md#the-no-poaching-gate) — an example phrase that
some skill's keywords already win is one that never fires in production.

You can write phrases naturally ("what's the time"); the loader normalises
them the same way it normalises user input, keeping `{slot}` intact.

### 3. The assistant

If no phrase matches either, the active assistant skill answers directly. One
can be active at a time; if none is, the user gets the didn't-understand
message.

## Manifest format

Ari uses [AgentSkills](https://agentskills.io/specification.md) as the
manifest envelope, with everything Ari-specific under `metadata.ari.*`.

AgentSkills was designed for LLM coding agents, and Ari can't *execute*
manifest prose — there's no LLM in the engine. So why use it?

1. **Standards alignment.** Every Ari skill is also a valid AgentSkills
   document, so authoring one in Claude Code, Cursor or Goose gets you that
   tool's skill discovery for free.
2. **Room to grow.** The markdown body is reserved as input for a richer
   on-device matcher. Same file, better matching, no migration.
3. **No format fork.** We follow the spec and use its official extension
   point rather than inventing another YAML schema.

Field reference: [reference-manifest.md](reference-manifest.md).

## Capabilities

A skill declares what it needs; each frontend declares what it provides; the
loader installs only if the skill's set is a subset of the host's.

Two kinds, and the distinction matters:

- **Host-import capabilities** (`http`, `storage_kv`, `location`, `authorize`,
  `media_services`, `tasks`, `calendar`) require a WASM import to exist. A
  host should claim these only once it's wired them up.
- **Frontend capabilities** (`notifications`, `launch_app`, `clipboard`,
  `tts`, `critical_alert`, `alarm`, `navigation`, `media_control`) need no
  import — they gate action-envelope slots, so declarative skills can use
  them.

At install time a **sneak guard** scans the compiled module's imports against
a static import→capability table and rejects any module importing something it
didn't declare.

`platforms` is an escape hatch for skills that genuinely can't work on an OS
even when capabilities match. Capabilities are the real contract.

Full list: [reference-capabilities.md](reference-capabilities.md).

## The sandbox

- **Memory** — `wasm.memory_limit_mb`, default 16, range 1–16. The ceiling is
  the 24-bit pointer field in the packed `execute` return value.
- **Fuel** — 50,000,000 units per call.
- **Isolation** — a fresh wasmtime store per call. No state survives; that's
  what `storage_kv` is for.

## Trust and signing

Bundles are tarballs of the skill directory, signed with Ed25519. The private
key is a GitHub Actions secret that nobody — maintainers included — can read
back out. The public key is compiled into the engine and rotates only with an
engine release.

Install:

1. Frontend asks the engine to install an id.
2. Engine fetches the bundle and signature from the registry release.
3. Verifies sha256 against `index.json`.
4. Verifies the Ed25519 signature against the baked-in key.
5. Extracts to the frontend's private skills directory, rejecting unsafe
   paths.
6. Parses `SKILL.en.md` plus any locale variants, validates `metadata.ari`,
   runs the capability check.
7. Registers the adapter.

Any failure aborts cleanly without touching engine state. Bundles are capped
at 8 MiB.

## Updates

On cold start — and daily via WorkManager on Android — the engine fetches
`index.json` and diffs it against installed versions. Anything newer that
satisfies the installed skill's engine semver range is downloaded, verified
and swapped in silently. Failures log and are ignored.

Auto-update is the deliberate default: a broken update on a voice assistant is
instantly visible to the user, while a missing update on a working skill is
invisible. The registry is signed, so the trust boundary is well defined.

## Repository layout

```
ari-skills/
├── docs/           # you are here
├── tools/          # validate, sideload-android, build-index.sh
├── templates/      # starter skills, validated and built in CI
├── sdk/
│   ├── rust/       # ari-skill-sdk
│   └── assemblyscript/
├── skills/         # one directory per published skill
├── bundles/        # generated — signed tarballs
├── manifests/      # generated
├── index.json      # generated — the catalogue
└── .github/workflows/
    ├── validate.yml          # PR gate
    └── sign-and-publish.yml  # on merge to main
```

Anything marked generated is written by CI. Never hand-edit it.

## Engine crates

```
ari-skill-loader/
├── lib.rs               # SkillLoader: install, uninstall, list, scan
├── manifest.rs          # frontmatter → typed structs
├── declarative.rs       # declarative adapter → impl Skill
├── wasm.rs              # wasmtime adapter + host imports + sneak guard
├── assistant.rs         # assistant-type manifests
├── scoring.rs           # PatternScorer
├── host_capabilities.rs # what the host provides
├── signature.rs         # Ed25519 verify
├── bundle.rs            # safe extraction
├── registry.rs          # index.json fetch + diff
└── localized_*.rs       # SKILL.{locale}.md and strings/
```

`ari-skill-validate` is a thin binary over the same loader, which is why the
validator can't drift from the engine.

## Deliberately out of scope

- Ratings, install counts, telemetry of any kind
- Skill-to-skill dependencies
- Per-skill runtime permission prompts — the install-time capability list is
  the only gate
- Paid skills and licence enforcement
- Machine-translated strings
- Wake-word and STT models as skills — those stay engine-managed
