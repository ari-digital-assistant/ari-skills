#!/usr/bin/env bash
#
# Build signed bundles and a fresh index.json for every skill under skills/.
#
# Used by .github/workflows/sign-and-publish.yml after a PR merges to main,
# and runnable locally to dry-run the publish step.
#
# For each skill it:
#   1. Runs ari-skill-validate --format=json to collect (id, version, name,
#      description, license) from the manifest. Validation failure aborts
#      the whole run (we never ship an unvalidated skill).
#   2. Packages skills/<slug>/ into bundles/<id>-<version>.tar.gz.
#   3. Signs the bundle with ari-sign-bundle using the key at $ARI_SIGNING_KEY_FILE.
#   4. Computes sha256.
#   5. Copies skills/<slug>/SKILL.md to manifests/<id>-<version>.md so clients
#      can fetch the full skill description (frontmatter + body) before
#      committing to an install, without downloading the whole bundle.
#   6. Writes index.json with one entry per skill.
#
# Required environment:
#   ARI_SIGNING_KEY_FILE   path to a 32-byte Ed25519 private key file (as
#                          produced by `ari-sign-bundle gen-key`)
#   ARI_SKILL_VALIDATE     (optional) path to the ari-skill-validate binary
#   ARI_SIGN_BUNDLE        (optional) path to the ari-sign-bundle binary
#
# If ARI_SKILL_VALIDATE / ARI_SIGN_BUNDLE aren't set, the script falls back
# to a sibling ari-engine checkout and runs the binaries via `cargo run`.

set -euo pipefail

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

: "${ARI_SIGNING_KEY_FILE:?ARI_SIGNING_KEY_FILE must be set to a private key file path}"

if [[ ! -f "$ARI_SIGNING_KEY_FILE" ]]; then
  echo "build-index: signing key file not found: $ARI_SIGNING_KEY_FILE" >&2
  exit 1
fi

# Resolve the two binaries we need.
resolve_binary() {
  local env_var="$1"
  local binary_name="$2"
  local env_value="${!env_var:-}"
  if [[ -n "$env_value" ]]; then
    echo "$env_value"
    return 0
  fi
  if command -v "$binary_name" >/dev/null 2>&1; then
    echo "$binary_name"
    return 0
  fi
  # Fall back to a sibling ari-engine checkout.
  for candidate in "$REPO_ROOT/../ari-engine" "$REPO_ROOT/../../ari-engine"; do
    if [[ -f "$candidate/Cargo.toml" ]]; then
      echo "cargo run --quiet --manifest-path $candidate/Cargo.toml -p $binary_name --"
      return 0
    fi
  done
  echo "build-index: could not locate $binary_name" >&2
  exit 2
}

VALIDATE=$(resolve_binary ARI_SKILL_VALIDATE ari-skill-validate)
SIGN=$(resolve_binary ARI_SIGN_BUNDLE ari-sign-bundle)

# jq is mandatory — the workflow runner has it, and so does any modern dev box.
if ! command -v jq >/dev/null 2>&1; then
  echo "build-index: jq is required but not installed" >&2
  exit 2
fi

echo "build-index: validating all skills under skills/ ..."
# shellcheck disable=SC2086
SKILL_JSON=$($VALIDATE --format=json skills/)

# Abort if any skill failed validation.
if echo "$SKILL_JSON" | jq -e 'any(.[]; .ok == false)' >/dev/null; then
  echo "build-index: one or more skills failed validation — refusing to publish" >&2
  echo "$SKILL_JSON" | jq -r '.[] | select(.ok == false) | "✗ \(.path): \(.failures | join("; "))"' >&2
  exit 1
fi

rm -rf bundles
mkdir -p bundles

# Wipe and recreate the manifests sidecar directory. Each entry is a verbatim
# copy of the skill's SKILL.md — frontmatter and body — so clients can render
# the full detail page without pulling the whole signed bundle.
rm -rf manifests
mkdir -p manifests

# Stream each skill through jq rather than a while-read loop this time —
# the validator JSON now carries arrays (capabilities, languages) which
# don't round-trip cleanly through TSV. We iterate with `jq -c '.[]'` and
# pipe each single-skill JSON object into a helper that does the bundle
# work and emits the index entry.
INDEX_TMP=$(mktemp)
trap 'rm -f "$INDEX_TMP"' EXIT

echo "$SKILL_JSON" | jq -c '.[]' | while read -r SKILL_ROW; do
  path=$(echo "$SKILL_ROW" | jq -r '.path')
  id=$(echo "$SKILL_ROW" | jq -r '.id // ""')
  version=$(echo "$SKILL_ROW" | jq -r '.version // ""')

  if [[ -z "$id" || -z "$version" ]]; then
    echo "build-index: skill at $path has no id/version — skipping" >&2
    continue
  fi

  slug=$(basename "$path")
  bundle_name="${id}-${version}.tar.gz"
  bundle_path="bundles/${bundle_name}"
  manifest_name="${id}-${version}.md"
  manifest_path="manifests/${manifest_name}"

  echo "build-index: packaging $id $version ($slug → $bundle_name)"
  # -C skills puts the archive root at <slug>/, which is what the engine's
  # bundle extractor expects.
  tar -czf "$bundle_path" -C skills "$slug"

  # shellcheck disable=SC2086
  $SIGN sign "$bundle_path" "$ARI_SIGNING_KEY_FILE" >/dev/null
  sha256_hex=$(cut -c1-64 <"${bundle_path}.sha256")

  # Copy SKILL.md out as a standalone sidecar so clients can preview the
  # full manifest (frontmatter + body) without fetching the bundle. The
  # source file has already been validated above, so no extra parsing
  # needed here — it's a byte-for-byte copy.
  cp "${path}/SKILL.md" "$manifest_path"

  # Build the index entry by augmenting the validator row with the
  # bundle paths we just produced. license / author / homepage come
  # from the validator as JSON-typed values (nullable strings), so we
  # pass them through verbatim rather than shoving them via --arg.
  echo "$SKILL_ROW" | jq \
    --arg bundle "$bundle_path" \
    --arg signature "${bundle_path}.sig" \
    --arg sha256 "$sha256_hex" \
    --arg manifest "$manifest_path" \
    '{
      id: .id,
      version: .version,
      name: .name,
      description: .description,
      license: .license,
      author: .author,
      homepage: .homepage,
      capabilities: (.capabilities // []),
      languages: (.languages // []),
      bundle: $bundle,
      signature: $signature,
      sha256: $sha256,
      manifest: $manifest
    }' \
    >>"$INDEX_TMP"
done

# Assemble index.json. generated_at is a UTC ISO-8601 timestamp; index_version
# lets us evolve the format without a flag-day migration.
jq -s --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  '{index_version: 1, generated_at: $ts, skills: .}' \
  "$INDEX_TMP" >index.json

COUNT=$(jq '.skills | length' index.json)
echo "build-index: wrote index.json with $COUNT skill(s)"
