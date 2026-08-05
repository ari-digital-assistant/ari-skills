---
name: chatgpt
license: MIT
description: >
  Use OpenAI's ChatGPT to answer general questions.
  Requires an API key from platform.openai.com.
  Your questions are sent to OpenAI's servers.
metadata:
  ari:
    id: dev.heyari.assistant.chatgpt
    version: "0.4.0"
    type: assistant
    author: Ari Project
    homepage: https://github.com/ari-digital-assistant/ari
    engine: ">=0.1"
    # GPT-5.4 family (and OpenAI's predecessors) are reliably multilingual
    # for the locales Ari ships against. The engine appends a per-request
    # "Please reply in <Language>." hint to the system prompt for any
    # non-English locale we don't ship a translated `system_prompt` for,
    # so adding a language here costs nothing on the skill side.
    languages: [en, it]
    examples:
      - text: "ask chatgpt why the sky is blue"
      - text: "ask chat gpt what the capital of France is"
      - text: "tell chatgpt to write me a poem"
      - text: "use chatgpt to summarise this"
      - text: "ask openai how photosynthesis works"
      - text: "chatgpt what's the weather like on Mars"
      - text: "hey ask chatgpt something for me"
      - text: "can you ask chatgpt to explain quantum computing"
      - text: "get chatgpt to help me with this"
      - text: "ask gpt what time zone Tokyo is in"
      - text: "use gpt to write a haiku"
      - text: "ask chat gpt for a joke"
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
      aliases: [chatgpt, chat gpt, gpt, openai]
      api:
        endpoint: https://api.openai.com/v1/chat/completions
        auth: bearer
        auth_config_key: api_key
        model_provider: openai
        tier_config_key: tier
        default_models:
          fast: gpt-5.6-luna
          balanced: gpt-5.6-terra
          smartest: gpt-5.6-sol
        # Only read by engines predating `tier_config_key`; those send
        # `temperature` unconditionally, so this must be a model that
        # accepts it. Tier-aware engines use `default_models` above.
        default_model: gpt-4.1-mini
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
Uses OpenAI's ChatGPT API to answer general knowledge questions.

You need an API key — get one at https://platform.openai.com/api-keys.
Queries are sent to OpenAI's servers; see their privacy policy for details.
