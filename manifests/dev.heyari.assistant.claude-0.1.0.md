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
    version: "0.1.0"
    type: assistant
    author: Ari Project
    homepage: https://github.com/ari-digital-assistant/ari
    engine: ">=0.1"
    languages: [en]
    examples:
      - text: "ask claude why the sky is blue"
      - text: "ask claude what the capital of France is"
      - text: "tell claude to write me a poem"
      - text: "use claude to summarise this"
      - text: "ask anthropic how photosynthesis works"
      - text: "claude what's the weather like on Mars"
      - text: "hey ask claude something for me"
      - text: "can you ask claude to explain quantum computing"
      - text: "get claude to help me with this"
      - text: "ask claude for a joke"
      - text: "use claude to draft an email"
      - text: "ask claude what the time is in Tokyo"
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
          - value: claude-opus-4-6
            label: Opus 4.6 (smartest, slower responses)
    assistant:
      provider: api
      privacy: cloud
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

You need an API key — get one at https://console.anthropic.com/settings/keys.
Queries are sent to Anthropic's servers; see their privacy policy for details.
