---
# `name` must match the directory (`home-assistant/`) — it's the stable
# system identifier, not a display field. Per-locale display strings
# live in `description` (below) and the markdown body. Don't translate
# this.
name: home-assistant
description: Controlla la tua casa intelligente con Home Assistant — accendi e spegni i dispositivi, imposta luminosità o temperatura, avvia le scene, controlla lo stato e chiedi dove si trovano le persone. Usa questa skill per qualsiasi richiesta di domotica o casa intelligente.
license: MIT
metadata:
  ari:
    id: dev.heyari.homeassistant
    version: "0.3.1"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.4"
    capabilities: [http, authorize, storage_kv]
    languages: [en, it]
    specificity: medium
    matching:
      # Pattern confrontati con l'input POST-NORMALIZZATO: minuscolo, con
      # apostrofi/elisioni sostituiti da uno spazio prima che l'engine
      # esegua la regex (`dov'è` → `dov è`). Gli accenti sopravvivono alla
      # normalizzazione, ma utenti e STT li omettono spesso: copriamo
      # entrambe le varianti (es. `dove? (e|è)`).
      #
      # A differenza dell'inglese, la normalizzazione italiana NON converte
      # i numeri scritti a lettere in cifre, quindi nessun pattern può
      # dipendere da `\d` ("imposta il soggiorno a ventuno gradi").
      patterns:
        # Imperativo — la forma di gran lunga più comune a voce.
        - regex: "\\b(accendi|spegni)\\b"
          weight: 0.9
        # Infinito, per le richieste cortesi ("puoi accendere le luci").
        - regex: "\\b(accendere|spegnere)\\b"
          weight: 0.9
        # Ancorato a luci/luminosità: da soli `abbassa`/`alza` sarebbero
        # troppo generici (es. "alza il volume").
        - regex: "\\b(abbassa|alza|attenua|aumenta|riduci) (le |la |il |l )?(luc[ei]|luminosit(a|à))\\b"
          weight: 0.85
        # `su` è escluso di proposito: "metti la musica su spotify" non è
        # una richiesta di domotica.
        - regex: "\\b(imposta|regola|metti) (il |la |lo |l |i |le )?.+ (a|al|alla)\\b"
          weight: 0.85
        # Include i participi (`aperta`, `chiuse`) perché in italiano le
        # domande sullo stato non riusano il verbo, a differenza
        # dell'inglese "is the garage door open".
        - regex: "\\b(apri|chiudi|blocca|sblocca|apert[oaie]|chius[oaie])\\b"
          weight: 0.8
        - regex: "\\b(attiva|avvia|esegui|lancia) (la |le )?scen[ae]\\b"
          weight: 0.9
        - regex: "\\bdove? (e|è|sono|si trova|si trovano)\\b"
          weight: 0.75
        # Nota: le keyword sono in AND (devono esserci tutte), come
        # nell'inglese `[thermostat, lights]`.
        - keywords: [termostato, luci]
          weight: 0.7
    examples:
      - text: "apri {cover}"
        weight: 0.95
      - text: "avvia la scena {scene}"
        weight: 0.95
      - text: "esegui la scena {scene}"
        weight: 0.95
      - text: "lancia la scena {scene}"
        weight: 0.95
      - text: "apri il portone"
        weight: 0.95
      - text: "accendi {entity}"
        weight: 0.95
      - text: "spegni {entity}"
        weight: 0.95
      - text: "puoi accendere {entity}"
        weight: 0.95
      - text: "puoi spegnere {entity}"
        weight: 0.95
      - text: "accendimi {entity}"
        weight: 0.95
      - text: "spegnimi {entity}"
        weight: 0.95
      - text: "vorrei che accendessi {entity}"
        weight: 0.95
      - text: "per favore accendi {entity}"
        weight: 0.95
      - text: "per favore spegni {entity}"
        weight: 0.75
      - text: "accendi {entity} grazie"
        weight: 0.95
      - text: "abbassa {entity}"
        weight: 0.95
      - text: "alza {entity}"
        weight: 0.95
      - text: "attenua {entity}"
        weight: 0.95
      - text: "abbassa {entity} al {percent} percento"
        weight: 0.95
      - text: "imposta {entity} al {percent} percento"
        weight: 0.75
      - text: "metti {entity} al {percent} percento"
        weight: 0.75
      - text: "riduci {entity} al {percent} percento"
        weight: 0.95
      - text: "chiudi {cover}"
        weight: 0.95
      - text: "puoi aprire {cover}"
        weight: 0.75
      - text: "puoi chiudere {cover}"
        weight: 0.95
      - text: "tira su {cover}"
        weight: 0.95
      - text: "tira giù {cover}"
        weight: 0.75
      - text: "imposta {room} a {temperature} gradi"
        weight: 0.75
      - text: "metti {room} a {temperature} gradi"
        weight: 0.75
      - text: "regola {room} a {temperature} gradi"
        weight: 0.95
      - text: "porta {room} a {temperature} gradi"
        weight: 0.95
      - text: "imposta il termostato a {temperature} gradi"
        weight: 0.95
      - text: "alza il riscaldamento"
        weight: 0.95
      - text: "abbassa il riscaldamento"
        weight: 0.95
      - text: "fa freddo alza il termostato a {temperature} gradi"
        weight: 0.95
      - text: "attiva la scena {scene}"
        weight: 0.95
      - text: "metti la scena {scene}"
        weight: 0.95
      - text: "la porta del garage è aperta"
        weight: 0.95
      - text: "la porta d'ingresso è chiusa"
        weight: 0.95
      - text: "è tutto spento in casa"
        weight: 0.95
      - text: "sono rimaste accese le luci"
        weight: 0.95
      - text: "chiudi a chiave la porta d'ingresso"
        weight: 0.95
      - text: "blocca la porta d'ingresso"
        weight: 0.95
      - text: "sblocca la porta d'ingresso"
        weight: 0.95
      - text: "dov'è {person}"
        weight: 0.95
      - text: "sai dov'è {person}"
        weight: 0.95
      - text: "mi dici dov'è {person}"
        weight: 0.95
      - text: "accendi {entity} per favore"
        weight: 0.95
      - text: "spegni tutto"
        weight: 0.95
      - text: "accendi tutte le luci"
        weight: 0.95
      - text: "spegni {entity} adesso"
        weight: 0.95
      - text: "accendi anche {entity}"
        weight: 0.95
      - text: "spegni pure {entity}"
        weight: 0.95
      - text: "accendi le luci della cucina"
        weight: 0.95
      - text: "spegni la lampada della camera"
        weight: 0.75
      - text: "imposta il soggiorno a 21 gradi"
        weight: 0.6
      - text: "abbassa le luci del corridoio al 30 percento"
        weight: 0.85
      - text: "attiva la scena serata film"
        weight: 0.85
      - text: "dov'è keith"
        weight: 0.95
    settings:
      - key: base_url
        label: "URL di Home Assistant"
        type: text
        required: true
      - key: sign_in
        label: "Accedi con Home Assistant"
        type: action
        depends_on: [base_url]
      - key: agent_id
        label: "Entità dell'agente di conversazione (lascia vuoto per usare l'agente predefinito o locale di Home Assistant)"
        type: dynamic_select
        required: false
        depends_on: [base_url]
      - key: token
        label: "Token di accesso a lunga durata"
        type: secret
        required: false
        validate: true
        depends_on: [base_url, token]
        collapsed_group: "Usa invece l'autenticazione con token"
        help_text: "Crea un token di accesso a lunga durata nel tuo profilo di Home Assistant (in fondo alla pagina) e incollalo qui."
    fallback:
      requires_setting: base_url
    wasm:
      module: skill.wasm
      memory_limit_mb: 2
---

# Home Assistant

Collega Ari al tuo server Home Assistant. Le frasi di comando ("accendi le
luci della cucina", "imposta la camera a 21", "attiva la scena serata film")
vengono inoltrate all'API `conversation/process` di HA, che risolve entità e
aree e risponde nella tua lingua. "Dov'è <persona>?" viene gestita leggendo
lo stato dell'entità `person.*` corrispondente. La posizione delle persone
viene letta direttamente, quindi funziona a prescindere da quali entità sono
esposte agli assistenti vocali.

**Configurazione:** inserisci l'URL del tuo server (ad es.
`http://homeassistant.local:8123` oppure il tuo indirizzo Nabu Casa) e un
token di accesso a lunga durata dalla pagina del tuo profilo HA. Un URL
`http://`, `.local` o con IP di rete locale funziona solo quando il
dispositivo è collegato alla rete di casa; per controllarla da fuori usa un
indirizzo Nabu Casa o un URL HTTPS esterno.
