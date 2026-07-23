# ari-skills

The community skill registry for the [Ari Digital Assistant](https://github.com/ari-digital-assistant/ari).

A **skill** turns a user utterance into a response — "what time is it?", "flip
a coin", "open Spotify". Ari ships a handful of built-in skills; this repo is
where everyone else's live.

## Write one

**→ [docs/](docs/) — start here.** The declarative tutorial takes about
fifteen minutes and needs no toolchain.

Three kinds of skill:

- **Declarative** — a `SKILL.en.md` manifest with patterns and responses. No
  code, no build step. This is what most skills should be.
- **WASM** — a sandboxed wasmtime module plus a manifest. For skills that need
  logic, state, HTTP or device capabilities.
- **Assistant** — a manifest describing an LLM API, for providing Ari's
  general-purpose brain.

The manifest format is [AgentSkills](https://agentskills.io)-compatible, with
Ari's config under `metadata.ari.*`. Every Ari skill is therefore also a valid
AgentSkills document, which is handy while you're authoring.

## Contribute

1. Read [docs/tutorial-declarative.md](docs/tutorial-declarative.md).
2. Fork, add your skill under `skills/<slug>/`.
3. `./tools/validate skills/<slug>/`
4. `./tools/sideload-android skills/<slug>/` to try it on a real device —
   exercises TTS, cards and alerts that CLI testing can't.
5. Open a pull request.

CI validates the manifest and checks your router examples don't collide with
another skill. A maintainer reviews. On merge the bundle is signed and
published, and every Ari user can install it from Settings → Skills → Browse
within minutes.

Full checklist and review criteria: [docs/publishing.md](docs/publishing.md).

## Repo layout

```
ari-skills/
├── docs/          # developer documentation
├── templates/     # starter skills, validated and built in CI
├── tools/         # validate, sideload-android
├── sdk/           # the Rust and AssemblyScript SDKs
├── skills/        # one directory per published skill
├── bundles/       # generated — signed tarballs
├── manifests/     # generated
└── index.json     # generated — the catalogue
```

Anything marked generated is written by CI. Don't hand-edit it.

## Governance

Reviewer responsibilities, branch protection and the trust model are in
[GOVERNANCE.md](GOVERNANCE.md). The short version:

- Skill PRs need one maintainer approval and a green `validate.yml`.
- Workflow changes under `.github/` get extra scrutiny — they're the
  load-bearing trust anchor.
- Maintainers can bypass approval on their own skill PRs. That's intentional:
  the validator and signing chain are the safety net, not the second human.
- The registry signing key is a GitHub Actions secret nobody can read back
  out.

## License

Each skill carries its own `license` field. The registry tooling and
scaffolding are licensed under [LICENSE](LICENSE).
