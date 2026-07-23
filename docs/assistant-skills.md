# Assistant skills

An **assistant skill** provides Ari's general-purpose brain — the thing that
answers when no specific skill can. ChatGPT, Claude, Gemini, a self-hosted
Ollama, anything with an HTTP API.

It's a manifest. No code, no WASM, no toolchain.

## How it differs from a normal skill

| | Normal skill | Assistant skill |
|---|---|---|
| Competes in the ranking rounds | Yes | **No** |
| When it runs | Its patterns matched | Nothing else claimed the utterance |
| How many can be active | All of them | **One** |
| Declares `matching` | Required | **Forbidden** |
| Declares `declarative`/`wasm` | One of them | **Forbidden** |
| Minimum 5 examples | Enforced | Exempt |

Set `type: assistant` and the manifest rules invert: no `matching`, no
`declarative`, no `wasm`, and an `assistant` block instead. Mixing them is a
parse error, so you'll find out immediately.

The user picks their assistant in Settings. If none is active, an unmatched
utterance gets "Sorry, I didn't understand that" — exactly as it does today
with "None" selected.

## A complete example

This is [`skills/claude`](../skills/claude), trimmed:

```markdown
---
name: claude
license: MIT
description: >
  Use Anthropic's Claude to answer general questions.
  Requires an API key from console.anthropic.com.
  Your questions are sent to Anthropic's servers.
metadata:
  ari:
    id: dev.heyari.assistant.claude
    version: "0.3.0"
    type: assistant
    author: Ari Project
    engine: ">=0.1"
    languages: [en, it]
    settings:
      - key: api_key
        label: API Key
        type: secret
        required: true
      - key: model
        label: Model
        type: select
        default: claude-sonnet-4-6
        options:
          - value: claude-haiku-4-5-20251001
            label: Haiku 4.5 (fastest, cheapest)
          - value: claude-sonnet-4-6
            label: Sonnet 4.6 (balanced)
    assistant:
      provider: api
      privacy: cloud
      aliases: [claude, anthropic]
      api:
        endpoint: https://api.anthropic.com/v1/messages
        auth: header
        auth_header: x-api-key
        auth_config_key: api_key
        model_config_key: model
        default_model: claude-sonnet-4-6
        request_format: anthropic
        api_version: "2023-06-01"
        api_version_header: anthropic-version
        system_prompt: >
          You are Ari, a helpful voice assistant.
          Answer the user's question in one short sentence.
        response_path: "content[0].text"
---
Uses Anthropic's Claude API to answer general knowledge questions.
```

## `metadata.ari.assistant`

| Field | Required | Notes |
|---|---|---|
| `provider` | **Yes** | `builtin` or `api`. Community skills use `api`. |
| `privacy` | **Yes** | `local` or `cloud`. Shown to the user in Settings. Be honest. |
| `api` | **Yes** when `provider: api` | See below. |
| `aliases` | No | Names the user can address directly — "ask claude…". |
| `config` | No | **Legacy.** Use the top-level `settings` instead. Declaring both is an error. |

### `api`

| Field | Required | Default | Notes |
|---|---|---|---|
| `endpoint` | One of these | — | Fixed URL. |
| `endpoint_config_key` | One of these | — | A settings key holding the URL — for self-hosted servers. |
| `default_endpoint` | No | — | Used when `endpoint_config_key` is empty. |
| `auth` | No | `none` | `bearer`, `header`, or `none`. |
| `auth_header` | With `auth: header` | — | Header name, e.g. `x-api-key`. |
| `auth_config_key` | No | — | Settings key holding the credential. |
| `model_config_key` | No | — | Settings key holding the model id. |
| `default_model` | **Yes** | — | Used when the user hasn't chosen. |
| `system_prompt` | **Yes** | — | String, or a `{locale: prompt}` map. |
| `request_format` | No | `openai` | `openai` or `anthropic`. |
| `response_path` | **Yes** | — | Dotted/indexed path to the answer text. |
| `api_version` | No | — | Extra version header value. |
| `api_version_header` | No | — | Its header name. |
| `max_tokens` | No | — | |
| `temperature` | No | — | |

### `request_format`

Two shapes are built in. `openai` covers OpenAI, Ollama, and the many APIs
that copied it; `anthropic` covers Claude.

Pick the one your provider speaks. If it speaks neither, it can't be an
assistant skill today — open an issue.

### `response_path`

Where the answer lives in the response JSON. Supports dots and array indices:

```yaml
response_path: "content[0].text"           # Anthropic
response_path: "choices[0].message.content" # OpenAI-shaped
```

### `system_prompt` per locale

Either a plain string (treated as English) or a map:

```yaml
        system_prompt:
          en: "You are Ari, a helpful voice assistant. Answer in one short sentence."
          it: "Sei Ari, un assistente vocale. Rispondi con una frase breve."
```

For any locale you *declare* in `languages` but don't write a prompt for, the
engine appends a "Please reply in <Language>." hint to the English prompt. So
with a genuinely multilingual model, adding a language can cost you nothing —
but only add it if you've checked the model actually handles it well.

**Keep the prompt short and demand short answers.** This is a voice assistant.
A model that returns three paragraphs produces thirty seconds of
text-to-speech that nobody wants.

## Settings and credentials

Declare fields in the top-level `metadata.ari.settings`, exactly as any other
skill would — see
[reference-manifest.md](reference-manifest.md#settings-fields).

Use `type: secret` for API keys. Secret fields are masked in the UI and routed
to encrypted storage automatically.

Point `auth_config_key` and `model_config_key` at those keys.

## Self-hosted providers

For something like Ollama, the endpoint is the user's, not yours:

```yaml
    settings:
      - key: server_url
        label: "Server URL"
        type: text
        required: true
        default: "http://localhost:11434"
        help_text: "Where your Ollama server is reachable."
    assistant:
      provider: api
      privacy: local
      api:
        endpoint_config_key: server_url
        default_endpoint: "http://localhost:11434/v1/chat/completions"
        auth: none
        default_model: llama3.2
        request_format: openai
        response_path: "choices[0].message.content"
        system_prompt: "You are Ari, a helpful voice assistant. Answer briefly."
```

`privacy: local` here is honest — nothing leaves the user's network. Don't
claim it for anything that does.

## Examples and aliases

Assistant skills are exempt from the five-example minimum, but examples still
help the router recognise "ask claude…" style direct addressing. Write them
around your `aliases`.

## Writing an honest `description`

The user reads this when choosing their assistant, so it needs to state:

1. What it is.
2. What it needs from them (an API key, a running server).
3. **Where their data goes.**

The Claude example does all three in three lines. Do the same. Anything less
on the third point will be sent back in review.

## Testing

```bash
./tools/validate skills/my-assistant
```

Then sideload it and pick it in Settings → Assistant. There's no CLI shortcut:
an assistant only runs when nothing else claims the utterance, so you need the
whole pipeline to exercise it.

## Publishing

Same flow as any skill: [publishing.md](publishing.md).

Expect closer review than usual. An assistant skill sees **every** utterance
that no skill matched — which is a lot of a user's private speech. The privacy
declaration and the description have to be exactly right.

## See also

- [`skills/claude`](../skills/claude), [`skills/chatgpt`](../skills/chatgpt), [`skills/gemini`](../skills/gemini) — the shipped examples
- [reference-manifest.md](reference-manifest.md) — the shared manifest fields
- `ari-engine/docs/assistant-skills.md` — the internal design document, if you want the engine side
