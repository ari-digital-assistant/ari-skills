# Governance

How the `ari-digital-assistant/ari-skills` registry is run, who can do what, and why the rules look the way they do.

## Roles

There are two roles that matter for the registry:

- **Contributor** — anyone who opens a PR adding or updating a skill. No persistent role; you become a contributor by submitting a PR and stop being one when it merges (or doesn't).
- **Maintainer** — has merge rights on `main` and admin rights on the repo. Reviews skill PRs, approves the ones that pass, and is responsible for upholding the trust model. The list of maintainers lives in [`.github/CODEOWNERS`](.github/CODEOWNERS) once that file exists.

There is no separate "publisher" role. **Merge to `main` *is* publication.** That's by design — the cryptographic signing happens automatically in `sign-and-publish.yml` after merge, with no human in the loop. Keeping these two events fused means you can't accidentally have an "approved but unpublished" or "published but unreviewed" state.

## The trust model in one paragraph

The registry has exactly **one** Ed25519 signing key. The private half lives as a GitHub Actions secret on this repo and is never accessible to any human, including maintainers. The public half is baked into the Ari engine binary at build time. Every bundle the registry publishes is signed with the private key by the `sign-and-publish.yml` workflow on merge to `main`. Every install on every Ari client verifies the signature against the baked-in public key before extracting a single byte. **The trust chain end-to-end is: user trusts the engine binary they installed → engine trusts the baked-in pubkey → pubkey verifies signatures from the registry CI → registry CI only runs on commits already merged to `main` → merges to `main` require maintainer review.** Every link in that chain is auditable.

For the full design, including threat model and what this *doesn't* protect against, read **[docs/skill-system.md](docs/skill-system.md#trust-and-signing)**.

## PR review responsibilities

When you (a maintainer) review a skill PR, you are the human gate in the trust chain. The validator catches mechanical problems; you catch judgement problems. Specifically, you are responsible for confirming:

1. **The description is honest.** What the skill *says* it does matches what its keywords + behaviour actually do.
2. **The keywords aren't greedy.** A skill that grabs `[the, a]` would hijack every utterance. The bot's PR comment shows the patterns — eyeball them.
3. **The specificity is right.** `high` is for narrow, confident matches. `low` is for catch-alls. Don't let a catch-all skill claim `high` to win the scoring round.
4. **The namespace claim is plausible.** `metadata.ari.id` is reverse-DNS. The contributor should have at least a token claim to the prefix. `dev.heyari.*` is reserved for the core team. Anyone else needs a domain or org they can defend.
5. **Capabilities match what's needed.** A skill declaring `[http]` should genuinely need HTTP. Over-asking is a red flag. The bot's PR comment lists declared caps and the WASM imports they correspond to — they should line up.
6. **No hidden lock-in.** A skill shouldn't hard-code dependencies on a specific proprietary service when a generic API exists. ([antislop rule 3](../antislop.md).)
7. **License is OSI-approved.** And explicitly stated in the manifest.

The PR template (when it exists) mirrors this checklist so contributors can self-check before submitting.

## Branch protection rules

Configured under Settings → Branches → Branch protection rules → `main`:

- **Require a pull request before merging:** ✓
- **Require approvals:** 1
- **Require status checks to pass before merging:** ✓
  - `validate.yml / validate-skills`
- **Require branches to be up to date before merging:** ✓
- **Do not allow bypassing the above settings:** ☐ (unchecked — see "Solo maintainer mode" below)
- **Restrict who can push to matching branches:** ✓ (maintainers + the `github-actions[bot]` only)

The bot is allowed to push to `main` *only* via the `sign-and-publish.yml` workflow, *only* to update `index.json`, and *only* in response to a merge that already passed review. The bot cannot open PRs, modify skill content, or run with elevated permissions in any other workflow.

### Workflow file changes

Modifications to anything under `.github/workflows/` warrant extra care because they're the load-bearing trust anchor: a malicious or buggy workflow could compromise the signing key. The convention is:

- **Always** open a PR for workflow changes. Don't push directly to `main`, even with bypass rights.
- **Read the diff carefully** before merging — workflow files are the one place where a typo can leak the signing key.
- If you're adding or modifying anything that touches `secrets.ARI_REGISTRY_SIGNING_KEY` (or any other secret), get a second pair of eyes if at all possible. Even if you have to wait a day.

The branch protection rules don't enforce these conventions on workflow files specifically — that's GitHub's limitation, not ours. Discipline > tooling here.

## Solo maintainer mode (current state)

As of writing, **the project has a single maintainer**. The branch protection rules above intentionally allow that maintainer to **bypass the approval requirement** for their own PRs (admin enforcement is off). This is on purpose and the trade-off is well-understood:

- **What we lose:** the "second pair of eyes" property for skill content. A solo maintainer reviewing their own PR is a single point of failure for human judgement errors.
- **What we keep:** every other safety mechanism in the chain. The validator still runs and must be green. The signing workflow still runs and signs only what's in `main`. The cryptographic chain to clients is unchanged. Capability declarations are still enforced at install time. Sneak guards still scan WASM imports. The rules still apply to *anyone who isn't a maintainer*.
- **Why it's the right call right now:** forcing a solo maintainer to perform a make-believe second-review ritual against themselves doesn't add safety — it adds friction without value, and friction in the publishing path is what kills small open-source projects. The cryptographic and CI safety nets are doing the load-bearing work.

When a second maintainer joins:

1. Add them to [`.github/CODEOWNERS`](.github/CODEOWNERS).
2. Update this document to remove the "solo maintainer mode" section.
3. **Optionally** turn on "Do not allow bypassing the above settings" so even maintainers go through the normal review flow. Recommended once there are ≥2 active maintainers; not required.
4. Consider raising "Require approvals" from 1 to 2 for high-trust paths (workflow files, `index.json` direct edits) using a CODEOWNERS rule.

Until then: when you (the solo maintainer) merge your own skill PR, **read the bot's PR comment** — that's the actually-useful safety net, not the approval requirement. The bot tells you what the skill declares, what its WASM imports require, and whether the capability promises line up. Five seconds of eyeballing that summary catches more real problems than any number of self-approvals ever would.

## Reporting a malicious or broken skill

If you find a published skill that's misbehaving, abusing capabilities, or impersonating another author:

1. **Open an issue** with the `bad-skill` label and the skill ID.
2. For active abuse (data theft, malware, etc.), email the maintainers directly — contact details in [SECURITY.md](SECURITY.md) once that exists.
3. Maintainers will assess and, if confirmed, **issue a takedown PR** that removes the skill from `skills/` and from `index.json`. The takedown lands the same way any other change does — via PR + signed re-publish.

Note that takedown only stops *future* installs. Users who already have the skill installed will continue to run it until their auto-update cycle picks up the new `index.json`, sees the skill is gone, and removes it locally. (That's a feature for step 7 — auto-update + auto-uninstall.) For acutely dangerous skills, we'd publish a coordinated security advisory alongside the takedown.

## Key rotation

The registry signing key is a security event, not a routine operation. Rotation procedure:

1. Generate a new Ed25519 keypair locally on a machine you trust.
2. Update the `ARI_REGISTRY_SIGNING_KEY` GitHub Actions secret with the new private key.
3. Re-run `sign-and-publish.yml` against `main` to re-sign every existing bundle with the new key.
4. Cut a new release of the Ari engine that has **both** the new and old pubkeys baked in (the new one as primary, the old one as deprecated).
5. After one engine release cycle has passed and most users have updated, cut another engine release that drops the old pubkey entirely.

This is the same pattern every signed-package system uses (apt, dnf, Homebrew). It's not glamorous. It works.

If the private key is *known* to be compromised (not just hypothetically lost), skip the slow rolling-rotation and ship an emergency engine update with the new key only. Users on the old engine are vulnerable until they update. There is no out-of-band emergency revocation channel and we don't intend to build one — the cost-benefit doesn't pencil out for a personal-assistant skill registry.
