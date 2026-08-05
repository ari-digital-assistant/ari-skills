---
name: claude
license: MIT
description: >
  Usa Claude di Anthropic per rispondere a domande generali. Richiede una chiave API da console.anthropic.com. Le tue domande vengono inviate ai server di Anthropic.
metadata:
  ari:
    id: dev.heyari.assistant.claude
    version: "0.4.0"
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
