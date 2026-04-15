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
    engine: ">=0.1"
    languages: [en]
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
    assistant:
      provider: api
      privacy: cloud
      api:
        endpoint: https://api.openai.com/v1/chat/completions
        auth: bearer
        auth_config_key: api_key
        model_config_key: model
        default_model: gpt-5.4-mini
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
          type: select
          default: gpt-5.4-mini
          options:
            - value: gpt-5.4-nano
              label: GPT-5.4 Nano (fastest, cheapest)
            - value: gpt-5.4-mini
              label: GPT-5.4 Mini (small, cost-efficient)
            - value: gpt-5.4
              label: GPT-5.4 (smartest, slower responses)
---
Uses OpenAI's ChatGPT API to answer general knowledge questions.

You need an API key — get one at https://platform.openai.com/api-keys.
Queries are sent to OpenAI's servers; see their privacy policy for details.
