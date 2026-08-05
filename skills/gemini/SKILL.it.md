---
name: gemini
license: MIT
description: >
  Usa Gemini di Google per rispondere a domande generali. Richiede una chiave API da aistudio.google.com. Le tue domande vengono inviate ai server di Google.
metadata:
  ari:
    id: dev.heyari.assistant.gemini
    version: "0.4.0"
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
