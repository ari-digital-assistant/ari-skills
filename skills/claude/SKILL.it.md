---
name: claude
license: MIT
description: >
  Usa Claude di Anthropic per rispondere a domande generali. Richiede una chiave API da console.anthropic.com. Le tue domande vengono inviate ai server di Anthropic.
metadata:
  ari:
    id: dev.heyari.assistant.claude
    version: "0.3.2"
    type: assistant
    author: Ari Project
    homepage: https://github.com/ari-digital-assistant/ari
    engine: ">=0.1"
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
      aliases: [claude, anthropic]
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
        response_path: "content[0].text"
---

# Claude (Italiano)

Usa l'API Claude di Anthropic per rispondere a domande di cultura generale.

Hai bisogno di una chiave API — ottienila su https://console.anthropic.com/settings/keys.
Le domande vengono inviate ai server di Anthropic; consulta la loro informativa sulla privacy per i dettagli.
