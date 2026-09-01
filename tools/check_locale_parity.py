#!/usr/bin/env python3
"""Every skill must ship what it says it ships.

`docs/i18n.md` states the invariant: **implemented == declared**. Nothing
ships a translation it doesn't declare, and nothing declares one it hasn't
written. A skill that gets this wrong fails in the two ways that are hardest
to spot from the outside:

  - declared but missing → the loader falls back to English at runtime, so
    an Italian user gets mixed-language chrome that reads worse than plain
    English would have.
  - written but undeclared → `index.json` is generated from the frontmatter,
    so the translation exists and nobody can find it.

Half-translated `strings/` tables fail the same way, one string at a time,
which is why the key sets and their `{placeholders}` are compared too.

Separately — and as a WARNING, never an error — this reports skills that are
English-only while the rest of the registry ships another language. That is
a perfectly legal thing to contribute, and refusing it would be a rotten
thing to do to someone writing their first skill. But it is also how
`dev.heyari.message` reached main on 2026-08-21 with no Italian manifest,
which meant it shipped with no Italian phrases to match against. Nothing
breaks on it, so this is a to-do list, not a gate.

Self-contained: stdlib plus PyYAML. No engine build, no cargo, no Rust.
"""

import json
import os
import re
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    sys.exit("ERROR: PyYAML is required (pip install pyyaml)")

ROOT = Path(__file__).resolve().parent.parent
SKILLS = ROOT / "skills"
PLACEHOLDER = re.compile(r"\{[a-z_][a-z0-9_]*\}", re.I)


def frontmatter(path: Path) -> dict:
    """The YAML block between the first two `---` fences."""
    text = path.read_text()
    if not text.startswith("---"):
        return {}
    parts = text.split("---", 2)
    if len(parts) < 3:
        return {}
    return yaml.safe_load(parts[1]) or {}


def declared_languages(skill_dir: Path) -> list:
    """`metadata.ari.languages` from the English manifest, which is the one
    every skill is required to have."""
    doc = frontmatter(skill_dir / "SKILL.en.md")
    ari = (doc.get("metadata") or {}).get("ari") or {}
    return list(ari.get("languages") or [])


def present_manifests(skill_dir: Path) -> set:
    return {p.name.split(".")[1] for p in skill_dir.glob("SKILL.*.md")}


def check_skill(skill_dir: Path, registry_locales: set) -> tuple:
    """Return (errors, warnings) for one skill directory."""
    errors, warnings = [], []
    name = skill_dir.name

    if not (skill_dir / "SKILL.en.md").is_file():
        # SKILL.md → SKILL.en.md is a rename, not a copy; the loader rejects
        # a directory holding both, so a bare SKILL.md is a stalled migration.
        errors.append(f"{name}: no SKILL.en.md (every skill needs one)")
        return errors, warnings

    declared = set(declared_languages(skill_dir))
    present = present_manifests(skill_dir)

    if not declared:
        errors.append(f"{name}: SKILL.en.md declares no `languages`")
        return errors, warnings

    for locale in sorted(declared - present):
        errors.append(f"{name}: declares `{locale}` but has no SKILL.{locale}.md")
    for locale in sorted(present - declared):
        errors.append(
            f"{name}: ships SKILL.{locale}.md but `languages` doesn't list `{locale}` "
            f"— index.json is built from the frontmatter, so nobody will find it"
        )

    # Each manifest must agree with the others about what the skill speaks.
    # Only en is read above, so a stale list in a sibling file would survive.
    for locale in sorted(declared & present):
        if locale == "en":
            continue
        doc = frontmatter(skill_dir / f"SKILL.{locale}.md")
        sibling = set(((doc.get("metadata") or {}).get("ari") or {}).get("languages") or [])
        if sibling != declared:
            errors.append(
                f"{name}: SKILL.{locale}.md declares languages {sorted(sibling)}, "
                f"SKILL.en.md declares {sorted(declared)}"
            )

    string_errors, string_warnings = check_strings(skill_dir, declared)
    errors += string_errors
    warnings += string_warnings

    missing = registry_locales - declared
    if missing:
        warnings.append(
            f"{name}: English-only ({', '.join(sorted(missing))} not declared) "
            f"— legal, but it's a translation gap"
        )

    return errors, warnings


def check_strings(skill_dir: Path, declared: set) -> tuple:
    """A `strings/` table must exist, and match, for every declared locale.

    A missing key falls back to English at runtime rather than failing, so
    nothing surfaces it except a user reading half-translated output.
    """
    strings = skill_dir / "strings"
    base = strings / "en.json"
    if not base.is_file():
        # Optional by design — a skill whose only output is an action
        # envelope has nothing to translate.
        return [], []

    errors, warnings = [], []
    name = skill_dir.name
    en = json.loads(base.read_text())

    for locale in sorted(declared):
        if locale == "en":
            continue
        path = strings / f"{locale}.json"
        if not path.is_file():
            errors.append(f"{name}: declares `{locale}` but has no strings/{locale}.json")
            continue
        table = json.loads(path.read_text())
        for key in sorted(set(en) - set(table)):
            errors.append(f"{name}: strings/{locale}.json is missing `{key}`")
        for key in sorted(set(table) - set(en)):
            errors.append(f"{name}: strings/{locale}.json has `{key}`, which en.json doesn't")
        for key in sorted(set(en) & set(table)):
            want = set(PLACEHOLDER.findall(en[key]))
            got = set(PLACEHOLDER.findall(table[key]))
            if want != got:
                # A WARNING, not an error: a locale legitimately reaches for
                # a different placeholder when the formatting convention
                # differs. weather's `time.hour` is "{h12}{ampm}" in English
                # and "le {h24}" in Italian, because Italy tells the time on
                # a 24-hour clock. Which placeholders a skill actually
                # supplies is decided in its code, and no amount of reading
                # JSON will reveal it — so this flags the typo case without
                # pretending to know the answer.
                warnings.append(
                    f"{name}: strings/{locale}.json `{key}` uses placeholders "
                    f"{sorted(got)}, en.json uses {sorted(want)} — deliberate?"
                )
    return errors, warnings


def main() -> int:
    dirs = sorted(d for d in SKILLS.iterdir() if d.is_dir() and not d.name.startswith("."))
    if not dirs:
        print(f"ERROR: no skills under {SKILLS}", file=sys.stderr)
        return 2

    # What the registry as a whole ships today, derived rather than hardcoded
    # so adding a third language doesn't mean remembering to edit this file.
    registry_locales = set()
    for d in dirs:
        registry_locales |= set(declared_languages(d))
    registry_locales.discard("en")

    errors, warnings = [], []
    for d in dirs:
        e, w = check_skill(d, registry_locales)
        errors += e
        warnings += w

    for w in warnings:
        print(f"::warning::{w}" if "GITHUB_ACTIONS" in os.environ else f"⚠ {w}")
    for e in errors:
        print(f"::error::{e}" if "GITHUB_ACTIONS" in os.environ else f"✗ {e}",
              file=sys.stderr)

    if errors:
        print(f"\n{len(errors)} locale-parity error(s) across {len(dirs)} skill(s).",
              file=sys.stderr)
        return 1
    print(f"✓ {len(dirs)} skill(s): every declared locale is implemented"
          f"{f' ({len(warnings)} warning(s))' if warnings else ''}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
