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
#   2. If ARI_REBUILD_SKILLS is set, rebuilds skill.wasm from source first (so
#      a stale committed binary can never ship), then packages skills/<slug>/
#      into bundles/<id>-<version>.tar.gz as a deterministic (reproducible)
#      archive. The tracked skill.wasm is restored afterwards.
#   3. Signs the bundle with ari-sign-bundle using the key at $ARI_SIGNING_KEY_FILE.
#   4. Computes sha256.
#   5. Copies skills/<slug>/SKILL.md to manifests/<id>-<version>.md so clients
#      can fetch the full skill description (frontmatter + body) before
#      committing to an install, without downloading the whole bundle.
#   6. Copies skills/<slug>/screenshots/<platform>/ to
#      screenshots/<id>-<version>/<platform>/ so detail pages can show
#      previews. Deliberately outside the bundle — see the tar step below.
#   7. Writes index.json with one entry per skill.
#
# Required environment:
#   ARI_SIGNING_KEY_FILE   path to a 32-byte Ed25519 private key file (as
#                          produced by `ari-sign-bundle gen-key`)
#   ARI_SKILL_VALIDATE     (optional) path to the ari-skill-validate binary
#   ARI_SIGN_BUNDLE        (optional) path to the ari-sign-bundle binary
#   ARI_REBUILD_SKILLS     (optional) if set, rebuild each skill's wasm from
#                          source before packaging (needs the wasm32 Rust
#                          toolchain; set by the publish workflow)
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

# Same treatment for the preview screenshots each skill ships under
# skills/<slug>/screenshots/<platform>/. They're browse-time decoration —
# the skill detail page in the app and on the website — so they publish as
# loose files here rather than going into the bundle, where every user
# installing the skill would pay to download pictures of it.
rm -rf screenshots
mkdir -p screenshots

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

  # Rebuild the wasm from source before packaging so a bundle can never ship a
  # stale binary: PR validation runs against the tracked skill.wasm but does
  # not rebuild it, so a source-only change could otherwise publish the
  # previous implementation. Gated behind ARI_REBUILD_SKILLS (the
  # sign-and-publish workflow sets it) so a local dry-run stays fast and does
  # not need the wasm toolchain. The tracked skill.wasm is backed up and
  # restored around the build — publishing must not mutate the source tree
  # (the workflow rebases before pushing) and a local run must stay read-only.
  wasm_backup=""
  if [[ -n "${ARI_REBUILD_SKILLS:-}" && -x "${path}/build.sh" ]]; then
    echo "  rebuilding ${slug}/skill.wasm from source"
    wasm_backup=$(mktemp)
    cp "${path}/skill.wasm" "$wasm_backup"
    ( cd "$path" && ./build.sh >/dev/null )
  fi

  # -C skills puts the archive root at <slug>/, which is what the engine's
  # bundle extractor expects.
  #
  # Exclude build-only directories: src/ (Rust sources + test fixtures) and
  # target/ (cargo output) are never needed at runtime — the engine loads
  # the prebuilt skill.wasm plus assets/, strings/ and the manifest. Shipping
  # them was dead weight; for the weather skill the src/ test fixture alone
  # was 60 KB, and historically nobody noticed because every other skill's
  # source is tiny. Skipping them keeps bundles lean and avoids leaking
  # source into the signed artifact.
  #
  # screenshots/ goes the same way, for the same reason: they're published
  # as loose files below and only ever fetched by a detail page, so putting
  # them in the bundle would charge every installing user for images they
  # have already seen — against an 8 MiB bundle ceiling.
  #
  # Deterministic archive: sorted entries, fixed mtime/ownership and `gzip -n`
  # (no embedded timestamp) so a bundle's bytes depend only on its contents.
  # Without this, the file mtimes from each fresh CI checkout leaked into the
  # tar headers and re-churned every bundle — and its signature — on every run.
  tar --sort=name --mtime='UTC 2020-01-01' --owner=0 --group=0 --numeric-owner \
    --exclude="$slug/src" --exclude="$slug/target" --exclude="$slug/screenshots" \
    -cf - -C skills "$slug" | gzip -n > "$bundle_path"

  if [[ -n "$wasm_backup" ]]; then
    cp "$wasm_backup" "${path}/skill.wasm"
    rm -f "$wasm_backup"
  fi

  # shellcheck disable=SC2086
  $SIGN sign "$bundle_path" "$ARI_SIGNING_KEY_FILE" >/dev/null
  sha256_hex=$(cut -c1-64 <"${bundle_path}.sha256")

  # Copy the canonical-locale manifest out as a standalone sidecar so
  # clients can preview the full manifest (frontmatter + body) without
  # fetching the bundle. Prefer SKILL.en.md (per-locale layout); fall
  # back to legacy SKILL.md for skills that haven't migrated. The
  # source file has already been validated above, so no extra parsing
  # needed here — it's a byte-for-byte copy.
  if [[ -f "${path}/SKILL.en.md" ]]; then
    cp "${path}/SKILL.en.md" "$manifest_path"
  else
    cp "${path}/SKILL.md" "$manifest_path"
  fi

  # Copy the preview screenshots out to screenshots/<id>-<version>/, keeping
  # the per-platform directories the validator already checked. Versioning
  # the directory means an update can change its screenshots without a
  # stale cached image from the previous version being served under the
  # same URL. The validator emits paths relative to the skill dir, and it
  # aborts the whole run on anything malformed, so everything listed here
  # is known-good by the time we copy it.
  shots_prefix="screenshots/${id}-${version}/"
  while read -r rel; do
    [[ -z "$rel" ]] && continue
    dest="${rel/#screenshots\//$shots_prefix}"
    mkdir -p "$(dirname "$dest")"
    cp "${path}/${rel}" "$dest"
  done < <(echo "$SKILL_ROW" | jq -r '(.screenshots // {}) | to_entries[] | .value[]')

  # Build the index entry by augmenting the validator row with the
  # bundle paths we just produced. license / author / homepage come
  # from the validator as JSON-typed values (nullable strings), so we
  # pass them through verbatim rather than shoving them via --arg.
  echo "$SKILL_ROW" | jq \
    --arg bundle "$bundle_path" \
    --arg signature "${bundle_path}.sig" \
    --arg sha256 "$sha256_hex" \
    --arg manifest "$manifest_path" \
    --arg shots_prefix "$shots_prefix" \
    '{
      id: .id,
      version: .version,
      name: .name,
      description: .description,
      type: (.type // "skill"),
      license: .license,
      author: .author,
      homepage: .homepage,
      capabilities: (.capabilities // []),
      languages: (.languages // []),
      bundle: $bundle,
      signature: $signature,
      sha256: $sha256,
      manifest: $manifest,
      localizations: (.localizations // {}),
      screenshots: ((.screenshots // {}) | map_values(map(sub("^screenshots/"; $shots_prefix))))
    }' \
    >>"$INDEX_TMP"
done

# Sign models.json alongside the bundles. It decides which model the cloud
# assistant skills actually call, so it needs the same Ed25519 guarantee they
# get — index.json itself is unsigned, and its sha256 is only a "the index
# lied" cross-check. Refreshed nightly by tools/build-models.sh via
# refresh-models.yml; signing stays here because this is the only place that
# holds the key.
if [[ ! -f models.json ]]; then
  echo "build-index: models.json is missing — run ./tools/build-models.sh" >&2
  exit 1
fi

echo "build-index: signing models.json"
# shellcheck disable=SC2086
$SIGN sign models.json "$ARI_SIGNING_KEY_FILE" >/dev/null
MODELS_SHA=$(cut -c1-64 <models.json.sha256)

# Assemble index.json. generated_at is a UTC ISO-8601 timestamp; index_version
# lets us evolve the format without a flag-day migration.
jq -s --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg models_sha "$MODELS_SHA" \
  '{
    index_version: 1,
    generated_at: $ts,
    models: {
      path: "models.json",
      signature: "models.json.sig",
      sha256: $models_sha
    },
    skills: .
  }' \
  "$INDEX_TMP" >index.json

COUNT=$(jq '.skills | length' index.json)
echo "build-index: wrote index.json with $COUNT skill(s) + signed models.json"
