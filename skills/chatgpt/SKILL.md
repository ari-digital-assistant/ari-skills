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
    version: "0.1.0"
    type: assistant
    author: Ari Project
    homepage: https://github.com/ari-digital-assistant/ari
    engine: ">=0.3"
    languages: [en]
    assistant:
      provider: api
      privacy: cloud
      api:
        endpoint: https://api.openai.com/v1/chat/completions
        auth: bearer
        auth_config_key: api_key
        model_config_key: model
        default_model: gpt-4o-mini
        system_prompt: >
          You are Ari, a helpful voice assistant.
          Answer the user's question in one short sentence.
        response_path: "choices[0].message.content"
      config:
        - key: api_key
          label: API Key
          type: secret
          required: true
        - key: model
          label: Model
          type: text
          default: gpt-4o-mini
---
Uses OpenAI's ChatGPT API to answer general knowledge questions.

You need an API key — get one at https://platform.openai.com/api-keys.
Queries are sent to OpenAI's servers; see their privacy policy for details.
