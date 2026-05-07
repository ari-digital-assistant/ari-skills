# ari-skills

The community skill registry for the [Ari Digital Assistant](https://github.com/ari-digital-assistant/ari).

A **skill** is a unit of behaviour that turns a user utterance into a response — "what time is it?", "flip a coin", "open Spotify". Ari ships a handful of built-in Rust skills out of the box; this repo is where everyone else's skills live.

Skills come in two flavours:

- **Declarative** — a `SKILL.md` manifest with keyword patterns and response templates. No code, no toolchain, just YAML and Markdown. This is what most skills should be.
- **WASM** — a sandboxed wasmtime module + manifest. For skills that need HTTP, storage, or non-trivial logic.

The manifest format is [AgentSkills](https://agentskills.io)-compatible, with Ari-specific config tucked under `metadata.ari.*` in the YAML frontmatter. That means every Ari skill is also a valid AgentSkills document and gets first-class support in Claude Code, Cursor, Goose, and other AgentSkills tools — useful while you're authoring.

## Documentation

- **[docs/skill-system.md](docs/skill-system.md)** — Technical overview of how the entire skill system works: the trait, scoring, the manifest format, capabilities, signing, registry workflow, engine integration. Read this if you want to understand the machinery.
- **[docs/skill-authors.md](docs/skill-authors.md)** — Quick guide for skill developers, with a full walkthrough of building and submitting a declarative skill. Read this if you just want to write one.
- **[docs/i18n.md](docs/i18n.md)** — How to add a non-English language to a skill: per-locale `SKILL.{locale}.md` files, `strings/{locale}.json` translation tables, and the SDK helpers (`t`, `get_locale`, `format_*`). Read this when you're ready to ship beyond English.

## Repo layout

```
ari-skills/
├── docs/                 # the two docs above
├── tools/                # local validator + helpers
├── skills/               # one directory per published skill
└── index.json            # generated catalogue (do not edit by hand)
```

## Contributing a skill

1. Read [docs/skill-authors.md](docs/skill-authors.md).
2. Fork this repo, add your skill under `skills/<slug>/`.
3. Run `./tools/validate skills/<slug>/`.
4. Iterate with `./tools/sideload-android skills/<slug>/` to try it on a real device/emulator before you open a PR — exercises TTS, UI, and action rendering that CLI testing can't.
5. Open a pull request.

CI validates the manifest. A maintainer reviews. On merge, the bundle is signed and published to the registry — and within minutes, every Ari user can install it from Settings → Skills → Browse.

## Governance

Reviewer responsibilities, branch protection, and the trust model are spelled out in **[GOVERNANCE.md](GOVERNANCE.md)**. The short version:

- All skill PRs need one maintainer approval and a green `validate.yml` before merge.
- Workflow file changes (under `.github/`) get extra scrutiny — they're the load-bearing trust anchor.
- Maintainers can bypass approval requirements for their own skill PRs (this is intentional; the validator + signing chain is the cryptographic safety net, not the second human).
- The registry signing key is a GitHub Actions secret; nobody, not even maintainers, can read it back out.

## License

Each skill carries its own `license` field in `SKILL.md`. The registry tooling and surrounding scaffolding are licensed under the terms in [LICENSE](LICENSE).
