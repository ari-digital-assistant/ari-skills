---
name: gemini
license: MIT
description: >
  Use Google's Gemini to answer general questions.
  Requires an API key from aistudio.google.com.
  Your questions are sent to Google's servers.
metadata:
  ari:
    id: dev.heyari.assistant.gemini
    version: "0.4.0"
    type: assistant
    author: Ari Project
    homepage: https://github.com/ari-digital-assistant/ari
    engine: ">=0.1"
    # Google's Gemini family is reliably multilingual for the locales
    # Ari ships against. The engine appends a per-request "Please reply
    # in <Language>." hint to the system prompt for any non-English
    # locale we don't ship a translated `system_prompt` for, so adding a
    # language here costs nothing on the skill side.
    languages: [en, it]
    examples:
      - text: "ask gemini why the sky is blue"
      - text: "ask gemini what the capital of France is"
      - text: "tell gemini to write me a poem"
      - text: "use gemini to summarise this"
      - text: "ask google how photosynthesis works"
      - text: "gemini what's the weather like on Mars"
      - text: "hey ask gemini something for me"
      - text: "can you ask gemini to explain quantum computing"
      - text: "get gemini to help me with this"
      - text: "ask gemini for a joke"
      - text: "use google ai to draft an email"
      - text: "ask gemini what the time is in Tokyo"
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
      aliases: [gemini, google ai]
      api:
        endpoint: https://generativelanguage.googleapis.com/v1beta/openai/chat/completions
        auth: bearer
        auth_config_key: api_key
        model_provider: google
        tier_config_key: tier
        default_models:
          fast: gemini-3.5-flash-lite
          balanced: gemini-3.6-flash
          smartest: gemini-3.1-pro-preview
        default_model: gemini-3.6-flash
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
        response_path: "choices[0].message.content"
---
Uses Google's Gemini API to answer general knowledge questions.

You need an API key — get one at https://aistudio.google.com/apikey.
Queries are sent to Google's servers; see their privacy policy for details.
