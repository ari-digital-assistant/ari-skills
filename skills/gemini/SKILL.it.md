---
name: gemini
license: MIT
description: >
  Usa Gemini di Google per rispondere a domande generali. Richiede una chiave API da aistudio.google.com. Le tue domande vengono inviate ai server di Google.
metadata:
  ari:
    id: dev.heyari.assistant.gemini
    version: "0.3.2"
    type: assistant
    author: Ari Project
    homepage: https://github.com/ari-digital-assistant/ari
    engine: ">=0.1"
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
      aliases: [gemini, google ai]
      api:
        endpoint: https://generativelanguage.googleapis.com/v1beta/openai/chat/completions
        auth: bearer
        auth_config_key: api_key
        model_config_key: model
        default_model: gemini-2.5-flash
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

# Gemini (Italiano)

Usa l'API Gemini di Google per rispondere a domande di cultura generale.

Hai bisogno di una chiave API — ottienila su https://aistudio.google.com/apikey.
Le domande vengono inviate ai server di Google; consulta la loro informativa sulla privacy per i dettagli.
