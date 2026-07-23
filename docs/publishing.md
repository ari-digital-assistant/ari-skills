# Publishing a skill

From "it works on my machine" to "every Ari user can install it".

## Before you open a PR

- [ ] `./tools/validate skills/<your-skill>` passes
- [ ] Your examples don't collide with another skill — see [the no-poaching gate](#the-no-poaching-gate)
- [ ] You tested inputs that **shouldn't** match, not just ones that should
- [ ] You tested on a device if your skill emits cards, alerts or notifications
- [ ] `capabilities` lists exactly what you use — no more
- [ ] `metadata.ari.id` is a reverse-DNS id you have a claim to
- [ ] Bundle is under 8 MiB
- [ ] `version` is bumped if you're updating an existing skill
- [ ] Any second language is one you actually speak

## Validate

```bash
./tools/validate skills/my-skill      # one skill
./tools/validate skills/              # the whole registry
```

This is a thin wrapper around `ari-skill-validate`, which reuses the engine's
own loader. There's no second implementation to drift, so anything it accepts,
the engine accepts.

It needs the binary. In order, it tries `$ARI_SKILL_VALIDATE`,
`ari-skill-validate` on `$PATH`, then a sibling `ari-engine` clone. Easiest
fix is to clone `ari-engine` next to `ari-skills`.

**What it fails on:** anything the loader rejects — bad names, missing
required fields, a `declarative`/`wasm` conflict, an out-of-range memory
limit, a `show_when` pointing at a field that doesn't exist, a WASM module
importing an undeclared capability.

**What it only warns about:** fewer than 5 examples, and a declarative
response that looks like a strings key but isn't in `strings/en.json`.

**What it does not check:** that your id is unique in the registry. Nothing
automated catches a collision — that's a reviewer's job, so pick a namespace
you actually control.

## Test locally

```bash
cd ../ari-engine
cargo run -p ari-cli -- --extra-skill-dir ../ari-skills/skills/my-skill "your test input"
```

Add `--debug` for the scoring trace: every skill's score, and which round the
winner won in.

Skills with capabilities need them granted explicitly — the CLI defaults to
the pure-frontend set:

```bash
cargo run -p ari-cli -- \
  --extra-skill-dir ../ari-skills/skills/my-skill \
  --host-capabilities http,storage_kv \
  --storage-dir /tmp/my-skill-storage \
  "your test input"
```

**Spend at least as long on negative tests.** A skill that fires when it
shouldn't is a worse bug than one that doesn't fire, because it steals
utterances from skills that would have handled them properly.

## Test on a device

The CLI prints envelope JSON. It cannot tell you whether TTS reads your
response naturally, whether your card looks right, whether an alert actually
rings, or whether `launch_app` resolves on a real phone.

```bash
./tools/sideload-android skills/my-skill
```

Rebuilds if there's a `build.sh`, validates, pushes over `adb` into the app's
private skills directory, and restarts the app so the engine rescans. Seconds
per iteration.

Requires a **debug** build installed (`run-as` doesn't work on release) and
`adb` on your `$PATH`. `--help` lists flags for an alternate package name,
device serial, and skipping the rebuild/validate/restart steps.

Useful while iterating:

```bash
adb logcat -s AriSkill                                        # your own ari::log output
adb logcat -s EngineModule AriEngine SkillUpdateWorker AssetResolver   # engine events
```

Do this before opening a PR for anything that renders UI. It's also the only
way to check that the router picks your skill up for the paraphrases in your
`examples`.

## The no-poaching gate

**This blocks merges and it surprises people, so read it properly.**

CI runs a test that takes every `examples[].text` in your PR and checks it
against **every other skill's keyword patterns**, in every locale. If another
skill's keywords win one of your examples, your PR fails.

### Why

The keyword matcher runs first and short-circuits. If skill B's keywords match
your example utterance, then in production that utterance goes to skill B and
the router is never consulted. So the example:

- teaches the router the opposite of what actually happens, and
- describes an utterance your skill can never serve anyway.

### What it looks like

```
### ❌ Router-example poaching

These examples are won by a different skill's keyword patterns.
  [en] "set a timer for five minutes" → won by dev.heyari.timer
```

### How to fix it

Either **re-word the example** so no keyword set wins it, or **tighten the
poaching skill's patterns** if they're genuinely over-broad.

The deeper fix is to write examples correctly in the first place:

> **Your examples should be the utterances your keywords MISS.**

The router is a fallback. Its training data should be the oblique, indirect,
conversational phrasings that a keyword list can never anticipate — not the
obvious ones your patterns already handle.

```yaml
# Wrong — your own keywords [roll, dice] already win these
examples:
  - text: "roll a dice"
  - text: "roll the dice"

# Right — no keyword set wins these, so the router has to
examples:
  - text: "settle this for me, one to six"
  - text: "i cannot find the dice, do it for me"
  - text: "what should i move on the board"
```

## Open the PR

```bash
git checkout -b my-skill
git add skills/my-skill
git commit -m "Add my-skill"
git push -u origin my-skill
```

CI runs the validator and the no-poaching test, and posts the result as a
comment. Both must be green.

## Review criteria

A maintainer reads for:

1. **Honesty.** Does the description match what the skill does? "Flips a
   coin" is fine. "AI-powered stochastic decision engine" is not.
2. **Namespace.** Is `metadata.ari.id` under a domain you have a claim to?
3. **Minimal capabilities.** Declaring `http` you never use gets flagged.
   Declaring `critical_alert` for something that isn't an alarm gets flagged
   harder.
4. **No duplication of a built-in.** If Ari already ships a calculator, don't
   publish a competing one — improve the built-in upstream.
5. **No third-party lock-in** where a generic API exists. "Open my podcast
   app" should use `launch_app` with a generic target, not hard-code one
   vendor's package name.
6. **Source-language only.** No machine translation, ever.
7. **Example quality.** Enough of them, realistic, and not poaching.

## After merge

The `sign-and-publish` workflow tarballs the directory, signs it with Ed25519,
uploads it as a release asset, and patches `index.json` on `main` via a bot
commit.

**Never hand-edit `index.json`, `bundles/` or `manifests/`.** They're
generated. A locally-signed copy will be overwritten, and committing one just
creates a conflict.

Within minutes your skill is installable from Settings → Skills → Browse.

## Updates

Bump `metadata.ari.version` and open another PR.

Once merged, installed copies update themselves — the engine diffs
`index.json` against what's installed on cold start, and daily in the
background on Android. Any update satisfying the user's engine semver range is
downloaded, signature-verified and swapped in silently. Failures log and are
otherwise ignored; they never block the app.

That's deliberate. A broken update on a voice assistant is instantly visible
— the user says something and gets nonsense. A *missing* update on a working
skill is invisible. Auto-updating is the lower-risk default, and the registry
is signed, so the trust boundary is well defined.

Practically: **your users get your bugs quickly, so test before you tag.**

## The trust model

Bundles are Ed25519-signed. The private key is a GitHub Actions secret nobody
can read back out; the public key is compiled into the engine and rotates only
with an engine release.

Install verifies the sha256 against `index.json`, then the signature against
the baked-in key, then extracts, parses, and runs the capability check. A
failure at any step aborts without touching engine state.

Reviewer responsibilities and branch protection: [GOVERNANCE.md](../GOVERNANCE.md).

## See also

- [troubleshooting.md](troubleshooting.md) — when validation or matching misbehaves
- [reference-manifest.md](reference-manifest.md#validation-errors) — what each validator error means
