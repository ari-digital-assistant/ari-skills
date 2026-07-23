# Security policy

This repo is the **skill registry** for the Ari digital assistant. Every skill
published here is signed and installed onto users' devices, so a compromise
here reaches real people. We take reports seriously and we'd rather hear about
something uncertain than not hear about it.

## Reporting a vulnerability

**Use GitHub's private vulnerability reporting:**
[Report a vulnerability](https://github.com/ari-digital-assistant/ari-skills/security/advisories/new)
— or the **Security** tab → **Report a vulnerability**.

That keeps the report private between you and the maintainers until there's a
fix, and lets us publish a coordinated advisory afterwards.

**Please don't open a public issue for a security problem.** Every published
skill auto-updates onto user devices, so a public report is a public exploit
window.

If you can't use GitHub's reporting for any reason, open a normal issue saying
only *"I have a security report, please get in touch"* — with no details — and
a maintainer will arrange a private channel.

### What to include

Whatever you have. Ideally:

- What the problem is and what an attacker could achieve with it.
- Steps to reproduce, or a proof of concept.
- The affected skill id and version, or the affected file.
- Whether it's already public anywhere.

### What to expect

The project currently has a **single maintainer** and no funded security
programme, so be realistic about response times:

| | Target |
|---|---|
| Acknowledgement | within 5 days |
| Initial assessment | within 14 days |
| Fix or mitigation for a confirmed critical issue | as fast as we can, prioritised over everything else |

We'll credit you in the advisory unless you'd rather we didn't. There is **no
bug bounty** — we can't afford one. If that changes, this file changes.

## Scope

### In scope

- **The signing and publishing chain.** Anything that could get an unsigned,
  altered or unreviewed bundle onto a user's device, or leak the registry
  signing key.
- **The validator and the capability model.** Anything that lets a skill do
  something it didn't declare — escaping the WASM sandbox, reaching a
  capability-gated host import without the capability, reading another skill's
  storage or assets.
- **Bundle handling.** Path traversal on extraction, hash or signature
  verification that can be bypassed, size-limit bypasses.
- **The registry tooling** in `tools/` and the workflows in `.github/`.
- **The SDKs** in `sdk/`, where a flaw would affect every skill built on them.

### Also worth reporting

- **A malicious or misbehaving published skill.** Abusing capabilities,
  exfiltrating data, impersonating another author. Report privately via the
  link above if it's active abuse; a normal issue is fine for something merely
  broken.

### Out of scope

- Vulnerabilities in the **Ari engine or frontends** — report those against
  [ari-engine](https://github.com/ari-digital-assistant/ari-engine) or the
  relevant frontend repo. If you're not sure which, report it here and we'll
  route it.
- Vulnerabilities in **third-party services** a skill talks to. Report those to
  the service.
- A skill requesting capabilities you think are excessive, where it does
  declare them honestly. That's a review-quality question — open a normal issue.

## What our threat model does and doesn't cover

Worth being straight about, so you know what counts as a finding.

**Covered:** an attacker cannot publish a skill without a maintainer merging
it, cannot alter a bundle after publication without invalidating its
signature, and cannot make a skill use a capability it didn't declare.

**Not covered:** a maintainer reviewing a skill badly. The human review step is
a judgement gate, not a cryptographic one, and a sufficiently subtle malicious
skill could pass it. That's a known limit, mitigated by the capability model
constraining what a skill can do even when it lies about intent.

**Also not covered:** there is no out-of-band emergency revocation channel, and
we don't intend to build one — the cost-benefit doesn't work for a registry
this size. Takedown stops future installs; existing installs are removed on the
next auto-update cycle.

The full design, including key rotation, is in
[GOVERNANCE.md](GOVERNANCE.md) and
[docs/internals.md](docs/internals.md#trust-and-signing).

## Handling a confirmed issue

1. Assess and confirm privately.
2. Fix, or take the offending skill down via PR — the takedown lands the same
   way any change does, through review and a signed re-publish.
3. Publish a GitHub security advisory, crediting the reporter.
4. If the registry signing key is implicated, follow the key rotation
   procedure in [GOVERNANCE.md](GOVERNANCE.md#key-rotation).

## Supported versions

The registry publishes from `main` only. There are no maintained release
branches, and there is no support for older `index.json` snapshots — clients
always fetch the current one.

Skills auto-update on users' devices, so **the current published version of
each skill is the only supported version**.
