# Ari Skills System — Technical Overview

This document describes how skills work in [Ari](https://github.com/ari-digital-assistant/ari), end to end. If you just want to write a skill, jump to [skill-authors.md](skill-authors.md) — this file is for people who need to understand the machinery underneath.

## What a skill is

A **skill** is a unit of behaviour that turns a user utterance into a response. "What time is it?" → `CurrentTimeSkill` → `"Half past four."`. "Open Spotify" → `OpenSkill` → `{"v":1,"launch_app":"Spotify"}` action envelope for the frontend to execute. Action envelopes carry a vocabulary of UI primitives (cards, alerts, notifications) and single-shot slots (`launch_app`, `search`, `open_url`, `clipboard`); skills declare *what* they want, frontends render it. Full reference: [action-responses.md](action-responses.md).

Ari has three flavours of skill, all implementing the same `Skill` trait at the engine boundary:

| Flavour | Where it lives | Who writes it | Runtime |
|---|---|---|---|
| **Built-in Rust** | `ari-engine/crates/ari-skills` | Ari core team | Compiled into the engine, always present |
| **Declarative** | `ari-digital-assistant/ari-skills` registry | Anyone | Pure manifest — patterns + response templates, no code |
| **WASM** | `ari-digital-assistant/ari-skills` registry | Anyone | Sandboxed wasmtime module + manifest |

The trait itself is dead simple:

```rust
pub trait Skill: Send + Sync {
    fn id(&self) -> &str;
    fn specificity(&self) -> Specificity;
    fn score(&self, input: &str, ctx: &SkillContext) -> f32;
    fn execute(&self, input: &str, ctx: &SkillContext) -> Response;
}
```

Declarative and WASM skills are wrapped by adapter structs (in the `ari-skill-loader` crate) that implement `Skill` and delegate to the manifest or the WASM module respectively.

## How a skill gets picked

Ari is **not** an LLM. Routing is deterministic. Every skill scores itself against the input, and the engine runs three ranking rounds with thresholds based on the skill's declared `Specificity`:

- Round 1: only `High` specificity skills, threshold 0.85
- Round 2: `High` + `Medium`, threshold 0.75
- Round 3: all skills, threshold 0.80

The first skill to clear its round's threshold wins. If nothing clears, Ari falls back to a default response. This means a `High`-specificity skill that's confident about a narrow input ("flip a coin") wins before a `Low`-specificity catch-all gets a look in.

For declarative skills, the engine builds a native scorer at load time from the manifest's `metadata.ari.matching.patterns` (keywords + optional regex + weights). **The WASM module is never called during scoring** — this is what lets the registry scale to hundreds of skills without paying an FFI cost per utterance.

WASM skills can opt in to custom scoring by setting `metadata.ari.matching.custom_score: true`. The loader then calls the module's exported `score(input, ctx) -> f32` during the ranking round. This is documented as a power-user feature and discouraged unless genuinely needed.

`execute()` is always called on the winning skill — declarative skills render their response template, WASM skills call into their `execute` export. Either way, exactly one call per utterance.

## Manifest format

Ari skills use [AgentSkills](https://agentskills.io/specification.md) `SKILL.md` files as their manifest envelope. AgentSkills is a format originally designed for LLM-based coding agents — Ari can't *execute* SKILL.md prose because there's no LLM in the engine, but the format is well-specified, widely adopted across the agent ecosystem, and explicitly invites client-specific extension via the `metadata` field. Ari's deterministic config lives entirely under `metadata.ari.*`.

Why bother? Three reasons:

1. **Standards alignment.** Every Ari skill is also a valid AgentSkills document. A developer authoring an Ari skill in Claude Code, Cursor, Goose, or any other AgentSkills-compatible tool gets that tool's skill discovery for free.
2. **Future LLM router.** The markdown body is reserved as input for a future on-device intent-classification model. Same file feeds the deterministic router today and a learned router tomorrow.
3. **No format fork.** We follow the spec exactly and use the official extension mechanism, rather than inventing yet another YAML/TOML schema.

A complete declarative manifest:

```markdown
---
name: coin-flip
description: Flips a virtual coin and returns heads or tails. Use when the user asks to flip a coin, toss a coin, or make a random binary choice.
license: MIT
metadata:
  ari:
    id: ai.example.coinflip
    version: "0.1.0"
    author: Someone <someone@example.com>
    homepage: https://example.com/coinflip
    engine: ">=0.3,<0.4"
    capabilities: []
    platforms: [android, linux]
    languages: [en]
    specificity: high
    matching:
      patterns:
        - keywords: [flip, coin]
          weight: 0.95
        - keywords: [toss, coin]
          weight: 0.95
    declarative:
      response_pick: ["Heads.", "Tails."]
---

# Coin Flip

Flips a virtual coin. Example: "flip a coin" → "Heads."
```

A WASM skill replaces the `declarative:` block with `wasm:`:

```yaml
    wasm:
      module: skill.wasm
      memory_limit_mb: 16
```

Exactly one of `declarative` or `wasm` must be present. Both is an error; neither is an error.

### Field reference

Standard AgentSkills frontmatter (top-level):

| Field | Required | Notes |
|---|---|---|
| `name` | Yes | Lowercase letters/digits/hyphens, ≤64 chars, must match the parent directory name |
| `description` | Yes | ≤1024 chars. Used by AgentSkills tooling and as fallback intent-router input |
| `license` | No | SPDX identifier or free text |
| `compatibility` | No | Free-text environment notes |
| `metadata` | No | Open key-value space; Ari uses `metadata.ari.*` |

Ari extensions under `metadata.ari`:

| Field | Required | Notes |
|---|---|---|
| `id` | Yes | Reverse-DNS unique identifier. Used for storage namespaces, signing, registry uniqueness |
| `version` | Yes | Semver |
| `author` | Yes | Free text |
| `homepage` | No | URL |
| `engine` | Yes | Semver range against the Ari engine version |
| `capabilities` | Yes (may be empty) | List of host capabilities the skill needs (see below) |
| `platforms` | No | Optional allowlist: `[android, linux, ...]`. Omit to allow all |
| `languages` | Yes | Source-language tags. Translations are *never* auto-generated |
| `specificity` | Yes | `low` / `medium` / `high` — feeds the ranking rounds |
| `matching.patterns` | Yes | List of `{keywords|regex, weight}` entries |
| `matching.custom_score` | No | WASM only. Default `false` |
| `declarative` | XOR `wasm` | Declarative response config |
| `wasm` | XOR `declarative` | `{module, memory_limit_mb}` |

## Capabilities

Capabilities are the contract between a skill and the host (engine + frontend). A skill declares what it needs; each frontend declares what it provides; the loader installs the skill only if the skill's set is a subset of the host's set.

| Capability | Meaning |
|---|---|
| `http` | Outbound HTTP from inside the WASM sandbox |
| `location` | Coarse or fine GPS |
| `notifications` | Show user-visible notifications |
| `launch_app` | Frontend-executed app launch (Android Intent / Linux xdg-open) |
| `clipboard` | Read/write clipboard |
| `tts` | Trigger TTS playback |
| `storage_kv` | Per-skill key-value scratch storage |

Declarative skills typically need no capabilities. WASM skills declare whatever they import from the host.

The optional `platforms` allowlist is an escape hatch for skills that genuinely can't run on a particular OS even when capabilities match. Capabilities are the primary contract; `platforms` is the override.

## Trust and signing

Bundles are tarballs of the skill directory, signed with Ed25519. The registry holds the private key as a GitHub Actions secret. The public key is baked into the engine at build time — rotation happens with an engine release.

Install flow:

1. Frontend asks the engine: "install skill `ai.example.coinflip`"
2. Engine fetches the bundle and signature from the registry release
3. Verifies sha256 against `index.json`
4. Verifies Ed25519 signature against the baked-in pubkey
5. Extracts to `<frontend filesDir>/skills/<id>/`
6. Parses `SKILL.md`, validates `metadata.ari`, runs capability check
7. Registers the adapter with `SkillLoader`

Any failure at any step aborts cleanly without touching engine state.

## Updates

On app cold start (and once daily via WorkManager on Android), the engine fetches `index.json` and diffs it against installed versions. Any update that satisfies the installed skill's semver range is silently downloaded, verified, and re-installed. Failed updates log but never block the app. The user sees a quiet "updated N skills" toast.

This is intentional. A broken skill update on a voice assistant is very visible — the user says something and gets nonsense — but a missing update on a working skill is invisible. Auto-updating is the lower-friction default and the registry is signed, so the trust boundary is well-defined.

## Registry

The `ari-digital-assistant/ari-skills` repo is the registry. Layout:

```
ari-skills/
├── README.md
├── docs/
│   ├── skill-system.md       # this file
│   └── skill-authors.md      # quick guide for skill developers
├── tools/
│   └── validate              # local validator
├── skills/
│   └── <slug>/
│       ├── SKILL.md
│       ├── skill.wasm        # if WASM
│       ├── assets/            # optional — bundled images, audio, etc.
│       │   ├── timer_icon.png
│       │   └── timer.mp3
│       └── references/       # optional, AgentSkills-style
├── index.json                # generated, machine-readable catalogue
├── signing/public.pem
└── .github/workflows/
    ├── validate.yml          # PR check
    └── sign-and-publish.yml  # on merge to main
```

Contribution flow:

1. Author forks the repo, creates a skill directory under `skills/`, opens a PR
2. `validate.yml` parses the manifest, checks AgentSkills naming rules, capability declarations, ID uniqueness against `index.json`, and runs a smoke test against a headless engine fixture (declarative) or wasmtime (WASM)
3. A maintainer reviews for honesty, quality, and namespace squatting
4. On merge, `sign-and-publish.yml` tarballs the directory, signs with Ed25519, uploads as a GitHub release asset, and patches `index.json` back into `main` via a bot commit

## Engine integration

The new crate is `ari-engine/crates/ari-skill-loader`:

```
ari-skill-loader/
├── lib.rs            # SkillLoader: install, uninstall, list, scan disk
├── manifest.rs       # serde structs for SKILL.md frontmatter
├── declarative.rs    # declarative adapter → impl Skill
├── wasm.rs           # wasmtime adapter → impl Skill
├── host_api.rs       # WASM host imports
├── capability.rs     # capability set + checks
├── signature.rs      # Ed25519 verify
└── registry.rs       # index.json fetch + diff
```

Existing crates touched lightly:

- `ari-core` — `SkillContext` gains capability handles. The `Skill` trait itself doesn't change.
- `ari-engine` — `Engine::new` takes a `SkillLoader` and merges built-in + loaded skills into the ranking pipeline.
- `ari-ffi` — new UniFFI exports: `list_installed_skills`, `fetch_registry`, `install_skill(id)`, `uninstall_skill(id)`, `check_updates`.

## What's deliberately out of scope

- Skill ratings, install counts, telemetry of any kind
- Skill-to-skill dependencies
- Per-skill runtime permission prompts (capability list at install time is the only gate)
- Paid skills, licensing enforcement
- Auto-translated strings — author provides source language only (per project rule)
- Wake-word and STT models as skills (those stay engine-managed)
