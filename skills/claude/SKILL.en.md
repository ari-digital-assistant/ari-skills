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
    version: "0.4.0"
    type: assistant
    author: Ari Project
    homepage: https://github.com/ari-digital-assistant/ari
    engine: ">=0.1"
    # Anthropic's Claude family is reliably multilingual for the locales
    # Ari ships against. The engine appends a per-request "Please reply
    # in <Language>." hint to the system prompt for any non-English
    # locale we don't ship a translated `system_prompt` for, so adding a
    # language here costs nothing on the skill side.
    languages: [en, it]
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
      - key: tier
        label: Model
        type: select
        default: balanced
        options:
          - value: fast
            label: Fast (quickest replies, lowest cost)
          - value: balanced
            label: Balanced
          - value: smartest
            label: Smartest (slowest replies, highest cost)
    assistant:
      provider: api
      privacy: cloud
      aliases: [claude, anthropic]
      api:
        endpoint: https://api.anthropic.com/v1/messages
        auth: header
        auth_header: x-api-key
        auth_config_key: api_key
        model_provider: anthropic
        tier_config_key: tier
        default_models:
          fast: claude-haiku-4-5
          balanced: claude-sonnet-5
          smartest: claude-opus-5
        # Only read by engines predating `tier_config_key`; those send
        # `temperature` unconditionally, so this must be a model that
        # accepts it. Tier-aware engines use `default_models` above.
        default_model: claude-sonnet-4-6
        request_format: anthropic
        api_version: "2023-06-01"
        api_version_header: anthropic-version
        system_prompt: >
          You are Ari, a helpful voice assistant. Answer the user's
          question in one short sentence. You have no access to live
          data (weather, news, prices), device or smart-home control,
          the user's location, or reminders, alarms, and timers —
          separately installed Ari skills handle those. If the user
          asks for one of them, say briefly that no installed skill
          handles it and that more skills are available in Ari's skill
          browser, and never imply you
          looked up live information. Asking follow-up questions to
          clarify what the user wants is fine. When a message contains
          structured instructions from an Ari skill, follow those
          instructions exactly.
        response_path: "content[0].text"
---
Uses Anthropic's Claude API to answer general knowledge questions.

You need an API key — get one at https://console.anthropic.com/settings/keys.
Queries are sent to Anthropic's servers; see their privacy policy for details.
