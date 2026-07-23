<!--
Thanks for contributing a skill.

This checklist mirrors what a maintainer reviews for (GOVERNANCE.md → PR
review responsibilities), so working through it first usually means a
one-round review instead of three.

New here? docs/tutorial-declarative.md walks the whole thing end to end.
-->

## What this skill does

<!-- One or two sentences. What does it do, and when should it fire? -->

## Type

- [ ] Declarative (manifest only)
- [ ] WASM
- [ ] Assistant
- [ ] Update to an existing skill

## Checklist

<!-- Full details for each of these are in docs/publishing.md. -->

- [ ] `./tools/validate skills/<slug>` passes
- [ ] I tested inputs that **shouldn't** match, not just ones that should
- [ ] `metadata.ari.id` is reverse-DNS under a namespace I have a claim to
      (`dev.heyari.*` is reserved for the core team)
- [ ] `capabilities` lists **exactly** what the skill uses — nothing
      speculative
- [ ] `description` honestly describes the behaviour, and its second sentence
      is written for the router
- [ ] `examples` are utterances my own keyword patterns **don't** match
      ([why](../blob/main/docs/publishing.md#the-no-poaching-gate))
- [ ] `license` is set and OSI-approved
- [ ] Any second language is one I actually speak — nothing machine-translated
- [ ] Bundle is under 8 MiB
- [ ] `metadata.ari.version` is bumped (updates only)

### If it emits cards, alerts or notifications

- [ ] I ran `./tools/sideload-android skills/<slug>` and checked it on a real
      device or emulator

### If it uses WASM

- [ ] `./build.sh` runs clean and `skill.wasm` is committed
- [ ] Declared capabilities and SDK feature flags agree

## Testing

<!--
What did you try? Paste a couple of utterances and what came back, including
one that should NOT have matched. `--debug` output is very welcome.
-->

## Anything else

<!--
Anything a reviewer should know: a capability that looks excessive but isn't,
a third-party service the skill depends on, a known limitation.
-->
