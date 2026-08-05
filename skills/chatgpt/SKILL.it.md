---
name: chatgpt
license: MIT
description: >
  Usa ChatGPT di OpenAI per rispondere a domande generali. Richiede una chiave API da platform.openai.com. Le tue domande vengono inviate ai server di OpenAI.
metadata:
  ari:
    id: dev.heyari.assistant.chatgpt
    version: "0.4.0"
    type: assistant
    author: Ari Project
    homepage: https://github.com/ari-digital-assistant/ari
    engine: ">=0.1"
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
            label: Veloce (risposte più rapide, costo minore)
          - value: balanced
            label: Bilanciato
          - value: smartest
            label: Più intelligente (risposte più lente, costo maggiore)
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
          Sei Ari, un assistente vocale utile. Rispondi alla domanda
          dell'utente con una frase breve. Non hai accesso a dati in
          tempo reale (meteo, notizie, prezzi), al controllo di
          dispositivi o della smart home, alla posizione dell'utente,
          né a promemoria, sveglie e timer — se ne occupano skill di
          Ari installate separatamente. Se l'utente chiede una di
          queste cose, digli brevemente che nessuna skill installata
          se ne occupa e che sono disponibili altre skill nel browser
          delle skill di Ari. Non lasciare mai intendere di aver
          consultato informazioni in tempo reale. Fare domande di
          chiarimento per capire cosa vuole l'utente va bene. Quando un
          messaggio contiene istruzioni strutturate da una skill di
          Ari, segui esattamente quelle istruzioni.
        response_path: "choices[0].message.content"
---

# ChatGPT (Italiano)

Usa l'API ChatGPT di OpenAI per rispondere a domande di cultura generale.

Hai bisogno di una chiave API — ottienila su https://platform.openai.com/api-keys.
Le domande vengono inviate ai server di OpenAI; consulta la loro informativa sulla privacy per i dettagli.
