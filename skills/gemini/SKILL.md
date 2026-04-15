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
    version: "0.1.0"
    type: assistant
    author: Ari Project
    homepage: https://github.com/ari-digital-assistant/ari
    engine: ">=0.1"
    languages: [en]
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
      - key: model
        label: Model
        type: select
        default: gemini-2.5-flash
        options:
          - value: gemini-2.5-flash
            label: Gemini 2.5 Flash (fastest, cheapest)
          - value: gemini-2.5-pro
            label: Gemini 2.5 Pro (smartest, slower responses)
    assistant:
      provider: api
      privacy: cloud
      api:
        endpoint: https://generativelanguage.googleapis.com/v1beta/openai/chat/completions
        auth: bearer
        auth_config_key: api_key
        model_config_key: model
        default_model: gemini-2.5-flash
        system_prompt: >
          You are Ari, a helpful voice assistant.
          Answer the user's question in one short sentence.
        response_path: "choices[0].message.content"
---
Uses Google's Gemini API to answer general knowledge questions.

You need an API key — get one at https://aistudio.google.com/apikey.
Queries are sent to Google's servers; see their privacy policy for details.
