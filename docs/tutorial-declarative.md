# Tutorial: your first skill

We're going to build a dice-rolling skill, test it, and get it ready to
submit. No compiler, no toolchain, no Rust. About fifteen minutes.

By the end you'll have a skill that answers "roll a dice" with "You rolled a
four." — and, more usefully, you'll understand why each line is there.

The finished result is in [`templates/dice-roll`](../templates/dice-roll) if
you'd rather read than type.

---

## What you need

- A GitHub account.
- A text editor.
- Optional but recommended: a clone of
  [ari-engine](https://github.com/ari-digital-assistant/ari-engine) next to
  this repo, so you can test locally. We'll cover the alternative if you
  don't have one.

## Step 1 — Fork and clone

Fork [ari-digital-assistant/ari-skills](https://github.com/ari-digital-assistant/ari-skills),
then:

```bash
git clone git@github.com:<your-user>/ari-skills.git
cd ari-skills
git checkout -b dice-roll
```

## Step 2 — Make the directory

```bash
mkdir -p skills/dice-roll
```

**The directory name matters.** It must exactly equal the `name:` field you're
about to write. That's an AgentSkills rule and the validator enforces it.
Lowercase letters, digits and hyphens only; no leading, trailing or doubled
hyphens.

## Step 3 — Write the manifest

Create `skills/dice-roll/SKILL.en.md`:

```markdown
---
name: dice-roll
description: Rolls a six-sided dice and returns the number. Use when the user asks to roll a dice, roll a die, throw the dice, or wants a random number between one and six.
license: MIT
metadata:
  ari:
    id: com.example.diceroll
    version: "0.1.0"
    author: Your Name <you@example.com>
    engine: ">=0.3,<0.4"
    capabilities: []
    languages: [en]
    specificity: high
    matching:
      patterns:
        - keywords: [roll, dice]
          weight: 0.95
        - keywords: [roll, die]
          weight: 0.95
        - keywords: [throw, dice]
          weight: 0.95
    examples:
      - text: "give me a number between one and six"
      - text: "i need a random number for the board game"
      - text: "pick a number, one to six"
      - text: "decide for me, one through six"
      - text: "what should i move on the board"
      - text: "settle this with a dice"
      - text: "i cannot find the dice, do it for me"
      - text: "random number please, six sided"
    declarative:
      response_pick: ["You rolled a one.", "You rolled a two.", "You rolled a three.", "You rolled a four.", "You rolled a five.", "You rolled a six."]
---

# Dice Roll

Rolls a six-sided dice. "roll a dice" → "You rolled a four."
```

That's a complete, publishable skill. Now let's go through the parts that
actually matter.

### `description` — this is not documentation

Two sentences, and each has a job:

1. **What the skill does.** "Rolls a six-sided dice and returns the number."
2. **When to use it**, packed with the words a real person might say. "Use
   when the user asks to roll a dice, roll a die, throw the dice, or wants a
   random number between one and six."

The router — a model — reads this to decide whether your skill fits an
utterance your keywords didn't catch. A vague description is a skill that
never gets picked. Write the second sentence as if you're describing the skill
to someone who has to recognise it from a stranger's phrasing, because that's
exactly the job.

### `matching.patterns` — the fast path

Each pattern is a set of `keywords` and a `weight`.

**Every keyword in a set must appear in the utterance** for that pattern to
match, as a **whole word**. Your skill's score is the **highest** weight among
its matching patterns — they don't add up.

So `keywords: [roll, dice]`:

| Utterance | Matches? | Why |
|---|---|---|
| "roll a dice" | ✅ 0.95 | both words present |
| "roll the dice for me" | ✅ 0.95 | both present, extra words are fine |
| "roll a die" | ❌ | "dice" absent — that's why we added a second pattern |
| "rolling dice" | ❌ | whole-word match; "rolling" ≠ "roll" |
| "dice" | ❌ | "roll" absent |

No stemming, no fuzzy matching. If you want "rolling" to work, list it.

**Patterns are matched against normalised text**, not raw speech. Before
matching, Ari lowercases the input, expands English contractions
("what's" → "what is"), strips punctuation, and turns number words into
digits. So never write an apostrophe or a capital letter in a keyword — it can
never match. Full rules: [troubleshooting.md](troubleshooting.md#my-pattern-never-matches).

### `specificity` — how confident you are

`high` means "I only fire on a narrow, unambiguous input." Combined with tight
keywords, it means a dice skill wins the round before some vague catch-all
gets a look in.

Use `high` when your patterns are specific. Use `low` for broad catch-alls.
If you leave it out entirely you get `medium`.

### `examples` — router training data, not documentation

This is the field everyone gets wrong, so read this bit twice.

Examples feed the on-device router's training set. The router is the
**fallback** layer — it only ever sees utterances the keyword matcher
**failed** to catch. Therefore:

> **A good example is one your own keyword patterns do NOT match.**

Look at the list above. Not one of them contains "roll" plus "dice". They're
all the oblique, human phrasings a keyword list can't anticipate — which is
precisely what the router is for.

If you write `- text: "roll a dice"`, your keywords already win that
utterance, the router will never be asked about it, and CI will **fail your
PR** for it. See [the no-poaching gate](publishing.md#the-no-poaching-gate).

Five is the minimum. Aim for 15–30.

### `declarative.response_pick`

One of these is chosen at random per invocation. Your options:

| Field | Behaviour |
|---|---|
| `response` | One fixed string |
| `response_pick` | A list; one picked at random each time |
| `response_template` | A string with `{{placeholders}}` |
| `action` | An action envelope — cards, alerts, app launches, etc. |

Exactly one of the three `response*` fields. `action` can accompany any of
them. See [reference-actions.md](reference-actions.md) for envelopes.

### `id`, `version`, `engine`

- `id` — a reverse-DNS identifier, unique across the registry. Use a domain
  you actually control. This is what storage and signing key off; it never
  changes.
- `version` — semver. Bump it every time you submit a change.
- `engine` — the range of Ari engine versions your skill works with.
  `">=0.3,<0.4"` is the current convention.

## Step 4 — Validate

```bash
./tools/validate skills/dice-roll
```

```
✓ skills/dice-roll: com.example.diceroll (8 examples)

validated 1 skill(s), 0 failure(s)
```

This runs the **exact same code the engine runs at install time** — it's not
a second implementation that can drift. If it passes here, the manifest is
structurally sound.

If you don't have an `ari-engine` clone nearby, the script tells you the three
other ways to get the validator. CI runs it on your PR either way.

## Step 5 — Test it for real

Validation proves the manifest parses. It doesn't prove the skill *behaves*.

With an `ari-engine` clone next door:

```bash
cd ../ari-engine
cargo run -p ari-cli -- --extra-skill-dir ../ari-skills/skills/dice-roll "roll a dice"
```

```
You rolled a four.
```

Now do the more important half — check it **doesn't** fire when it shouldn't:

```bash
cargo run -p ari-cli -- --extra-skill-dir ../ari-skills/skills/dice-roll "what time is it"
cargo run -p ari-cli -- --extra-skill-dir ../ari-skills/skills/dice-roll "roll the pastry"
```

Both should decline. A skill that fires on everything is worse than no skill.

Add `--debug` to see the scoring trace and find out exactly which pattern won
and with what score.

### Testing on a real device

The CLI can't tell you whether TTS reads your response naturally, or whether a
card renders properly. For that, push your working tree straight into a debug
build of the Android app:

```bash
./tools/sideload-android skills/dice-roll
```

It validates, pushes over `adb`, and restarts the app so the engine rescans.
Takes seconds. See [publishing.md](publishing.md#test-on-a-device) for the
requirements and the useful `logcat` filters.

## Step 6 — Make it translatable

You don't have to, but it's cheap now and annoying later.

Move the text into a strings table and reference it by key:

```yaml
    declarative:
      response_pick:
        - dice.one
        - dice.two
        - dice.three
        - dice.four
        - dice.five
        - dice.six
```

`skills/dice-roll/strings/en.json`:

```json
{
  "dice.one": "You rolled a one.",
  "dice.two": "You rolled a two.",
  "dice.three": "You rolled a three.",
  "dice.four": "You rolled a four.",
  "dice.five": "You rolled a five.",
  "dice.six": "You rolled a six."
}
```

At execute time each response string is looked up in the table for the user's
language. A value that isn't a key is emitted verbatim, so this is safe to
adopt gradually.

**Only ship languages you actually speak.** Adding a locale later is a new
`SKILL.it.md` plus a `strings/it.json` — nothing else changes. See
[i18n.md](i18n.md).

## Step 7 — Submit

```bash
git add skills/dice-roll
git commit -m "Add dice-roll skill"
git push -u origin dice-roll
```

Open a PR against `ari-digital-assistant/ari-skills`. CI validates the
manifest and checks your examples against every other skill's patterns. A
maintainer reviews. On merge, the bundle is signed and published — and every
Ari user can install it from Settings → Skills → Browse within minutes.

Full checklist, review criteria and what happens after merge:
[publishing.md](publishing.md).

---

## Where next

- Your skill needs logic, state or network access → [tutorial-wasm.md](tutorial-wasm.md)
- You want to return a card instead of a sentence → [reference-actions.md](reference-actions.md)
- Something isn't matching and you can't see why → [troubleshooting.md](troubleshooting.md)
- You want every manifest field, not just the ones used here → [reference-manifest.md](reference-manifest.md)
