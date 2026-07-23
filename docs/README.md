# Ari skill developer docs

Everything you need to write, test and publish a skill.

## Start here

**Never written an Ari skill?** Do the declarative tutorial. It takes about
fifteen minutes, needs no toolchain, and ends with a skill you could submit.

| I want to… | Read |
|---|---|
| Write my first skill, no code | [tutorial-declarative.md](tutorial-declarative.md) |
| Write a skill with logic, state or network access | [tutorial-wasm.md](tutorial-wasm.md) |
| Ship an alternative "brain" for Ari (ChatGPT, Ollama, …) | [assistant-skills.md](assistant-skills.md) |
| Look up a manifest field | [reference-manifest.md](reference-manifest.md) |
| Find out what my skill is allowed to do | [reference-capabilities.md](reference-capabilities.md) |
| Show a card, fire an alert, launch an app, ask a follow-up | [reference-actions.md](reference-actions.md) |
| Look up an SDK function | [reference-sdk.md](reference-sdk.md) |
| Add a second language | [i18n.md](i18n.md) |
| Get my skill into the registry | [publishing.md](publishing.md) |
| Work out why the bloody thing isn't matching | [troubleshooting.md](troubleshooting.md) |
| Understand how any of this actually works | [internals.md](internals.md) |
| Use AssemblyScript instead of Rust | [assemblyscript.md](assemblyscript.md) |

## The five-minute mental model

**A skill turns one utterance into one response.** "flip a coin" → "Heads."
That's the whole contract. No sessions, no conversation state, no framework.

### There are three kinds

| Kind | You write | Use it when |
|---|---|---|
| **Declarative** | A manifest. That's it. | The answer is a fixed string, a random pick, or a template. Most skills. |
| **WASM** | A manifest + a sandboxed module | You need logic, persistent state, HTTP, or the device's calendar/location/… |
| **Assistant** | A manifest describing an LLM API | You're providing a general-purpose brain, not a specific behaviour. |

Every skill is a directory with a `SKILL.en.md` manifest inside it. That file
is a valid [AgentSkills](https://agentskills.io) document — standard
frontmatter on top, Ari's config under `metadata.ari.*`.

### How your skill gets chosen

Two layers, in order:

1. **The keyword matcher.** Deterministic, instant, free. Every skill scores
   itself against the utterance using the patterns you declared. Best score
   over the threshold wins and runs. Most utterances stop here.
2. **The router.** Only runs when nothing matched. An on-device model reads
   your `description` and decides whether your skill fits. This is the safety
   net for phrasings your keywords missed.

If neither claims it, the user's configured assistant answers instead.

This split is the single most important thing to understand, because **the two
layers read completely different fields**:

| Layer | Reads | Tune it by |
|---|---|---|
| Keyword matcher | `matching.patterns` | Adding tight keyword sets |
| Router | `description`, and it was trained on your `examples` | Writing a rich description and realistic examples |

A consequence that catches everyone: **your `examples` should be the
utterances your keywords *miss*.** An example your own patterns already win is
one the router never sees in production, and CI will reject it. See
[publishing.md](publishing.md#the-no-poaching-gate).

### What your skill can send back

Plain text covers most skills. When you need more, return an **action
envelope** — a JSON object asking the frontend to render a card, ring an
alert, post a notification, launch an app, copy to the clipboard, start
navigation, or ask the user a follow-up question.

You declare *what* you want; each frontend renders it its own way. No
platform-specific code ever lives in a skill. Full vocabulary:
[reference-actions.md](reference-actions.md).

## Templates

Four working starters, all validated and built in CI. Copy one and rename it.

| Template | Kind | Shows |
|---|---|---|
| [`templates/dice-roll`](../templates/dice-roll) | Declarative | Patterns, random responses, translatable strings |
| [`templates/tally`](../templates/tally) | Rust WASM | A capability, user settings, a stat card, `t()` |
| [`templates/countdown`](../templates/countdown) | Rust WASM | A live countdown card with an alert on completion |
| [`templates/echo-as`](../templates/echo-as) | AssemblyScript | The minimum viable non-Rust skill |

## House rules

Short version — the full list is in [publishing.md](publishing.md#review-criteria).

1. **Be honest in your description.** "Flips a coin", not "AI-powered
   stochastic decision engine".
2. **Source language only.** Ship the language you actually speak. Never
   machine-translate.
3. **Don't squat namespaces.** Use a reverse-DNS id you have a claim to.
4. **Declare exactly the capabilities you use.** No speculative `http`.
5. **Don't reinvent a built-in.** Improve it upstream instead.
6. **No hard-coded third-party lock-in** where a generic API exists.

## Getting help

- Issues: <https://github.com/ari-digital-assistant/ari-skills/issues>
- Main project: <https://github.com/ari-digital-assistant/ari>
- AgentSkills spec: <https://agentskills.io/specification.md>
