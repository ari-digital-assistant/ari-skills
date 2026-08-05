#!/usr/bin/env bash
#
# Resolve the current fast / balanced / smartest model for each cloud
# assistant provider and write models.json at the repo root.
#
# Used by .github/workflows/refresh-models.yml on a nightly schedule, and
# runnable locally to preview what tomorrow's refresh would land.
#
# WHY THIS EXISTS
#   The cloud assistant skills (chatgpt, claude, gemini) used to enumerate
#   concrete model IDs in their `settings` block, so every provider release
#   meant a skill version bump plus re-translating option labels. Instead the
#   skills now offer three stable tiers and the engine resolves tier -> model
#   ID through this file, falling back to the manifest's pinned default when
#   the file is missing (first run, offline, fetch failure).
#
# HOW A TIER IS CHOSEN
#   Primary path is the provider's own naming: OpenAI's luna/terra/sol (and
#   the older nano/mini/pro), Anthropic's haiku/sonnet/opus, Google's
#   flash-lite/flash/pro. Among models matching a tier's hint we take the
#   most recently released. When a provider renames its lineup out from under
#   the hints, we fall back to output-price bands — first within the newest
#   release date that offers at least three distinct prices, then across
#   everything released within 18 months of that provider's newest model.
#   Each pick carries selection_method + confidence so a reviewer can see
#   which path produced it.
#
#   Preview and experimental models are INCLUDED by default. Google in
#   particular ships its frontier Pro models under a `-preview` suffix for a
#   long time — as of this writing every Gemini 3.x Pro is preview-only, so
#   excluding them drops `smartest` back to gemini-2.5-pro from June 2025,
#   which is older and weaker than the Flash model in `balanced`. The label
#   doesn't mean "don't use in production" the way it does elsewhere.
#   Set INCLUDE_PREVIEW=0 to restrict to models with no preview marker.
#
#   The `preview` boolean on each tier records which way a pick went, so the
#   nightly refresh PR shows it. Previews do get withdrawn eventually
#   (gemini-3-pro-preview already was), and the per-tier pins in each skill's
#   manifest track these picks including previews — so when one is withdrawn,
#   fix the pin in the same release that notices. The pin is the fallback for a
#   device that has never fetched the catalog, so a dead pin there means a dead
#   tier until the skill ships again.
#
# OMIT_PARAMS
#   models.dev tracks a per-model `temperature` boolean. Newer reasoning
#   families reject sampling parameters outright with HTTP 400 rather than
#   ignoring them — as of this writing that is all three OpenAI tiers plus
#   Anthropic's Sonnet 5 and Opus 5. Each tier therefore carries the list of
#   request fields the engine must leave out. Absent or false both mean omit:
#   sending an unsupported sampling param is a hard failure, whereas omitting
#   a supported one just means provider-default sampling.
#
# Usage:
#   ./tools/build-models.sh                        # fetch models.dev, write models.json
#   ./tools/build-models.sh ./fixture.json         # build from a local fixture
#   INCLUDE_PREVIEW=0 ./tools/build-models.sh      # stable-marked models only
#
# Requires: curl, jq 1.6+

set -euo pipefail

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

SOURCE="${1:-https://models.dev/api.json}"
INCLUDE_PREVIEW="${INCLUDE_PREVIEW:-1}"

case "$INCLUDE_PREVIEW" in
  0|1) ;;
  *)
    echo "build-models: INCLUDE_PREVIEW must be 0 or 1" >&2
    exit 2
    ;;
esac

if ! command -v jq >/dev/null 2>&1; then
  echo "build-models: jq is required but not installed" >&2
  exit 2
fi

if [[ "$SOURCE" =~ ^https?:// ]]; then
  if ! command -v curl >/dev/null 2>&1; then
    echo "build-models: curl is required but not installed" >&2
    exit 2
  fi
  fetch=(curl -fsSL --retry 3 --retry-delay 1 "$SOURCE")
else
  if [[ ! -r "$SOURCE" ]]; then
    echo "build-models: cannot read source: $SOURCE" >&2
    exit 2
  fi
  fetch=(cat "$SOURCE")
fi

MODELS_TMP=$(mktemp)
trap 'rm -f "$MODELS_TMP"' EXIT

"${fetch[@]}" | jq \
  --arg generated_at "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
  --arg source "$SOURCE" \
  --argjson include_preview "$INCLUDE_PREVIEW" '

  def model_text:
    (((.id // "") + " " + (.name // "")) | ascii_downcase);

  def date_epoch($date):
    ($date // "1970-01-01") as $d
    | try (
        ($d
          + (if ($d | length) == 4 then "-01-01"
             elif ($d | length) == 7 then "-01"
             else ""
             end)
          + "T00:00:00Z")
        | fromdateiso8601
      ) catch 0;

  # Prefer explicit, undated IDs over moving "latest" aliases and dated
  # snapshots. Tested against .id alone: the date pattern is end-anchored,
  # and model_text ends with the display name, so matching against that
  # never fired and dated snapshots won on map order instead.
  def id_preference:
    ((.id // "") | ascii_downcase) as $id
    | if ($id | test("latest")) then -2
      elif ($id | test("[-_](20[0-9]{2}[-_]?[0-9]{2}[-_]?[0-9]{2}|20[0-9]{6})$")) then -1
      else 0
      end;

  def is_preview:
    ((.status // "") | ascii_downcase | test("alpha|beta"))
    or (model_text | test("preview|experimental"));

  # Anything other than an explicit `true` means omit. models.dev may not
  # have annotated a brand-new model yet, and guessing wrong in the
  # permissive direction is a 400 on every request.
  def omit_params:
    if (.temperature == true) then [] else ["temperature", "top_p", "top_k"] end;

  def is_general_text_model($provider):
    ((.status // "") != "deprecated")
    and (((.modalities.input // ["text"]) | index("text")) != null)
    and (((.modalities.output // ["text"]) | index("text")) != null)
    and ((.cost.output? | type) == "number")
    and (($include_preview == 1) or (is_preview | not))
    and (
      model_text
      | test("embedding|moderation|realtime|audio|speech|transcrib|tts|image|video|imagen|veo|live|translate|search|deep[- ]?research|computer[- ]?use")
      | not
    )
    and (if $provider == "anthropic" then (.id | test("^claude-")) else true end)
    and (if $provider == "google" then (.id | test("^gemini-")) else true end)
    and (if $provider == "openai" then (.id | test("^(gpt-|o[0-9])")) else true end);

  def candidates($provider):
    (.[$provider].models // {})
    | to_entries
    | map(.value + {id: .key})
    | map(select(is_general_text_model($provider)));

  def hints($provider; $tier):
    if $provider == "openai" and $tier == "fast" then
      {positive: "luna|nano", negative: ""}
    elif $provider == "openai" and $tier == "balanced" then
      {positive: "terra|mini", negative: ""}
    elif $provider == "openai" and $tier == "smartest" then
      {positive: "sol|pro", negative: ""}
    elif $provider == "anthropic" and $tier == "fast" then
      {positive: "haiku", negative: ""}
    elif $provider == "anthropic" and $tier == "balanced" then
      {positive: "sonnet", negative: ""}
    elif $provider == "anthropic" and $tier == "smartest" then
      {positive: "opus|fable", negative: ""}
    elif $provider == "google" and $tier == "fast" then
      {positive: "flash[- ]?lite|lite", negative: ""}
    elif $provider == "google" and $tier == "balanced" then
      {positive: "flash", negative: "flash[- ]?lite|lite"}
    elif $provider == "google" and $tier == "smartest" then
      {positive: "pro|ultra", negative: ""}
    else
      {positive: "a^", negative: ""}
    end;

  def pick_named($models; $positive; $negative):
    [
      $models[]
      | select(model_text | test($positive))
      | select(($negative == "") or (model_text | test($negative) | not))
    ]
    | if length == 0 then null
      else
        sort_by(
          (.release_date // ""),
          (.last_updated // ""),
          id_preference,
          (.cost.output // 0)
        )
        | last
        | . + {
            selection_method: "semantic-name",
            confidence: "high"
          }
      end;

  def unique_price_models($models):
    $models
    | sort_by(.cost.output)
    | group_by(.cost.output)
    | map(
        sort_by(
          (.release_date // ""),
          (.last_updated // ""),
          id_preference
        )
        | last
      );

  # Best generic fallback: a new release date containing at least three price tiers.
  def newest_three_price_cohort($models):
    $models
    | sort_by(.release_date // "")
    | group_by(.release_date // "")
    | map(
        unique_price_models(.) as $priced
        | select(($priced | length) >= 3)
        | {
            date: (.[0].release_date // ""),
            models: $priced
          }
      )
    | if length == 0 then null else sort_by(.date) | last end;

  # Last resort: use distinct price bands among models released in the last 18 months
  # relative to that providers newest candidate.
  def recent_price_pool($models):
    [$models[] | . + {selection_epoch: date_epoch(.release_date)}] as $dated
    | ($dated | map(.selection_epoch) | max // 0) as $newest
    | [
        $dated[]
        | select(($newest - .selection_epoch) <= (548 * 86400))
      ]
    | unique_price_models(.);

  def pick_from_price_pool($pool; $tier):
    ($pool | sort_by(.cost.output)) as $sorted
    | if ($sorted | length) == 0 then null
      elif $tier == "fast" then $sorted[0]
      elif $tier == "smartest" then $sorted[-1]
      else $sorted[(($sorted | length) / 2 | floor)]
      end;

  def pick_fallback($models; $tier):
    newest_three_price_cohort($models) as $cohort
    | if $cohort != null then
        pick_from_price_pool($cohort.models; $tier)
        | . + {
            selection_method: "same-release-price-cohort",
            confidence: "medium"
          }
      else
        recent_price_pool($models) as $pool
        | pick_from_price_pool($pool; $tier)
        | if . == null then null
          else . + {
            selection_method: "recent-price-band",
            confidence: "low"
          }
          end
      end;

  def choose($models; $provider; $tier):
    hints($provider; $tier) as $h
    | pick_named($models; $h.positive; $h.negative) as $named
    | if $named != null then $named else pick_fallback($models; $tier) end;

  def public_model($model):
    if $model == null then null
    else {
      id: $model.id,
      name: ($model.name // $model.id),
      family: ($model.family // null),
      release_date: ($model.release_date // null),
      last_updated: ($model.last_updated // null),
      preview: ($model | is_preview),
      omit_params: ($model | omit_params),
      pricing_per_million_tokens: {
        input: ($model.cost.input // null),
        output: ($model.cost.output // null)
      },
      context_tokens: ($model.limit.context // null),
      selection_method: $model.selection_method,
      confidence: $model.confidence
    }
    end;

  def provider_result($provider):
    candidates($provider) as $models
    | {
        fast: public_model(choose($models; $provider; "fast")),
        balanced: public_model(choose($models; $provider; "balanced")),
        smartest: public_model(choose($models; $provider; "smartest"))
      };

  {
    schema_version: 1,
    generated_at: $generated_at,
    source: $source,
    include_preview: ($include_preview == 1),
    providers: {
      openai: provider_result("openai"),
      anthropic: provider_result("anthropic"),
      google: provider_result("google")
    }
  }
' >"$MODELS_TMP"

# An unresolved tier means the engine silently falls back to a manifest pin
# that nobody has looked at in months. Refuse to publish it.
MISSING=$(jq -r '
  .providers
  | to_entries[]
  | .key as $p
  | .value
  | to_entries[]
  | select(.value == null)
  | "  \($p)/\(.key)"' "$MODELS_TMP")

if [[ -n "$MISSING" ]]; then
  echo "build-models: no model resolved for:" >&2
  echo "$MISSING" >&2
  echo "build-models: refusing to write models.json — check the tier hints" >&2
  exit 1
fi

mv "$MODELS_TMP" models.json
trap - EXIT

jq -r '
  .providers
  | to_entries[]
  | .key as $p
  | .value
  | to_entries[]
  | "build-models: \($p)/\(.key) -> \(.value.id)"
    + (if (.value.omit_params | length) > 0 then " (omits \(.value.omit_params | join(", ")))" else "" end)
    + (if .value.confidence != "high" then " [\(.value.selection_method), \(.value.confidence) confidence]" else "" end)
' models.json
