# Writing an Ari Skill

A short, practical guide. For the full architecture, see [skill-system.md](skill-system.md).

## What you need to know first

- A skill is a folder containing a `SKILL.md` file. Optionally a WASM module, an icon, and reference docs.
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

### 3. Write `SKILL.md`

Create `skills/coin-flip/SKILL.md`:

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
    declarative:
      response_pick: ["Heads.", "Tails."]
---

# Coin Flip

Flips a virtual coin. Example: "flip a coin" → "Heads."
```

A few things worth pointing at:

- **`name` and the directory name must match.** AgentSkills rule. Lowercase, hyphens, no leading/trailing/consecutive hyphens.
- **`description` is two sentences for a reason.** First sentence: what the skill does. Second sentence: when to use it. AgentSkills-compatible LLM tools (and Ari's future learned router) read this to decide whether to activate the skill. A vague description means a skill that never wins.
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

### 6. Test against a local engine

If you have the Ari engine checked out:

```bash
cd ../ari/ari-engine
cargo run -p ari-cli -- \
  --extra-skill-dir ../../ari-skills/skills/coin-flip \
  "flip a coin"
```

You should see `Heads.` or `Tails.` Try a few inputs that *shouldn't* match — `"what time is it"`, `"flip the pancakes"` — and confirm your skill stays out of it.

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
| `action` | A JSON object the frontend will execute (e.g. open an app, set an alarm). Combine with `response_template` to give the user verbal feedback. |

Example combining a template and an action:

```yaml
    declarative:
      response_template: "Opening {{target}}."
      action:
        type: launch_app
        target: "{{target}}"
```

The capture mechanism for `{{target}}` from the input is documented in [skill-system.md](skill-system.md).

## When you actually need WASM

Reach for WASM only if your skill needs to:

- Make HTTP calls (weather, news, public APIs)
- Do non-trivial parsing of the user input
- Maintain state across invocations (`storage_kv`)
- Compute something the declarative templates can't express

A WASM skill declares its capabilities and ships a `skill.wasm` module alongside `SKILL.md`. The host exposes a tiny API: `log`, `http_fetch`, `storage_get`, `storage_set`, `get_capability`. That's it. If you need more, file an issue first — the surface is intentionally small.

Two SDKs are available:

- **Rust** — `sdk/rust/` and template at `templates/rust/`
- **AssemblyScript** — `sdk/assemblyscript/` and template at `templates/assemblyscript/`

Copy a template, edit SKILL.md + the source file, run `build.sh`, and you're done. Full docs: [wasm-sdk.md](wasm-sdk.md).

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

### Example utterances (future)

Built-in Rust skills in `ari-engine` declare a list of training utterances
via the `Skill::example_utterances()` trait method, paired with the JSON
arguments the function call should produce. These feed directly into the
FunctionGemma fine-tuning dataset.

Community SKILL.md skills don't have a trait, but a future enhancement will
pull example utterances from a `## Example utterances` markdown section in
the SKILL.md body. **You can include this section now** — it's already a
loose convention in the reference skills, and it'll start contributing to
training data once the community-skill extractor lands.

```markdown
## Example utterances

- "flip a coin"
- "toss a coin please"
- "heads or tails"
- "let's leave it to chance"
- "should I or shouldn't I, flip a coin"
```

Aim for 20-30 varied phrasings. Cover paraphrases, indirect language,
conversational filler ("can you", "please", "I need"). The point is to
teach the router that all these natural utterances should land on your
skill, not just the rigid ones your keyword patterns catch.

## Rules of the road

These will save your PR from review friction:

1. **Be honest in the description.** "Flips a coin" is fine. "AI-powered randomness engine for decision-making" is not.
2. **Source language only.** Don't auto-translate strings. Declare `languages: [en]` (or whatever you actually wrote it in) and stop. Translations belong in a translation platform, not in your PR.
3. **Don't squat namespaces.** Pick a `metadata.ari.id` reverse-DNS prefix you actually control or have a clear claim to.
4. **Tight keywords beat clever regex.** A short keyword list with `specificity: high` will outperform an over-eager regex that fires on every other utterance.
5. **Declare exactly the capabilities you need.** Asking for `http` when you don't use it will get flagged in review.
6. **Don't reinvent built-ins.** If Ari already ships a `CalculatorSkill`, don't publish a competing one — improve the built-in via a PR to the main Ari repo instead.
7. **No third-party-specific lock-in when a generic API exists.** "Open my podcast app" should use `launch_app` with a generic target, not hard-code one app's package name.

## Where to ask questions

- Issues: <https://github.com/ari-digital-assistant/ari-skills/issues>
- Main project: <https://github.com/ari-digital-assistant/ari>
- Spec for the underlying AgentSkills format: <https://agentskills.io/specification.md>
