---
# `name` deve corrispondere alla directory (`message/`) — è
# l'identificatore di sistema stabile, non un campo di visualizzazione.
# Le stringhe localizzate stanno in `description` (qui sotto) e nel corpo
# markdown. Non tradurre questo campo.
name: message
description: >
  Manda un messaggio a qualcuno via SMS, WhatsApp, Telegram, Signal,
  Messenger, Slack, Matrix o email. Dici a chi e cosa, e Ari lo invia
  oppure apre l'app con il messaggio già pronto.
license: MIT
metadata:
  ari:
    id: dev.heyari.message
    version: "0.2.0"
    type: skill
    author: Ari Project
    homepage: https://github.com/ari-digital-assistant/ari-skills
    license: MIT
    engine: ">=0.3"
    languages: [en, it]
    capabilities: [send_message, contacts, http, reply]
    specificity: high
    matching:
      # `custom_score` è attivo, quindi il motore chiama la export `score`
      # del modulo e questi pattern non vengono mai eseguiti. Restano
      # accurati perché documentano le forme che il parser accetta, e il
      # validatore richiede almeno una voce.
      #
      # Perché lo scoring custom, in italiano: il parser pretende una
      # preposizione fra il verbo e il nome ("scrivi *a* Mario", "scrivi
      # *alla* mamma). È quella preposizione a impedire che "scrivi una
      # poesia" o "di che colore è il cielo" finiscano qui, e una regex non
      # può esprimere il vincolo senza lookaround — che il crate regex di
      # Rust non ha. I verbi che reggono l'oggetto diretto (`avvisa`,
      # `avverti`, `contatta`) sono l'eccezione: sono già specifici da soli.
      custom_score: true
      patterns:
        - regex: "\\b(manda|invia|scrivi) (un|una) (messaggio|sms|mail|email|whatsapp|telegram)\\b"
          weight: 0.95
        - regex: "\\b(avvisa|avverti|contatta)\\b"
          weight: 0.9
        - regex: "\\bfai sapere a\\b"
          weight: 0.9
        - regex: "\\brispondi a\\b"
          weight: 0.9
    settings:
      - key: confirm_before_sending
        label: Prima di inviare
        type: select
        default: always
        options:
          - value: always
            label: Rileggilo e chiedi
          - value: never
            label: Invia subito
      - key: matrix_homeserver
        label: Server Matrix
        type: text
      - key: matrix_token
        label: Token di accesso Matrix
        type: secret
      - key: default_service
        label: Manda i messaggi con
        type: select
        default: sms
        options:
          - value: sms
            label: Messaggi
          - value: whatsapp
            label: WhatsApp
          - value: telegram
            label: Telegram
          - value: signal
            label: Signal
          - value: messenger
            label: Messenger
          - value: slack
            label: Slack
          - value: matrix
            label: Matrix
          - value: email
            label: Email
    # Il testo degli esempi è POST-NORMALIZZAZIONE: minuscolo, con le
    # elisioni sostituite da uno spazio ("di'" → "di", "l'incontro" →
    # "l incontro"). Gli accenti sopravvivono alla normalizzazione e vanno
    # scritti.
    examples:
      # Forme canoniche: le vince il parser della skill, quindi in
      # produzione il router non le vede mai. Stanno qui perché
      # documentano gli slot — `recipient`, `body`, `service`.
      - text: "scrivi a mario che arrivo tardi"
        args: '{"recipient":"mario","body":"arrivo tardi"}'
      - text: "manda un messaggio a gail dicendo che sono in ritardo"
        args: '{"recipient":"gail","body":"sono in ritardo"}'
      - text: "di a mario che torno presto"
        args: '{"recipient":"mario","body":"torno presto"}'
      - text: "avvisa gail che faccio tardi"
        args: '{"recipient":"gail","body":"faccio tardi"}'
      - text: "fai sapere a gail che sono sull autobus"
        args: '{"recipient":"gail","body":"sono sull''autobus"}'
      - text: "manda un sms a gail ci vediamo alle 8"
        args: '{"recipient":"gail","body":"ci vediamo alle 8","service":"sms"}'
      - text: "manda una mail a gail ricordati del latte"
        args: '{"recipient":"gail","body":"ricordati del latte","service":"email"}'
      - text: "scrivi a mario buon compleanno su whatsapp"
        args: '{"recipient":"mario","body":"buon compleanno","service":"whatsapp"}'
      # Frasi oblique che il parser qui sopra non intercetta di proposito:
      # sono quelle che il router vede davvero in produzione.
      - text: "puoi dire a mario che sono in ritardo"
        args: '{"recipient":"mario","body":"sono in ritardo"}'
      - text: "vorrei mandare un messaggio a gail"
        args: '{"recipient":"gail"}'
      - text: "devo avvisare mario che faccio tardi"
        args: '{"recipient":"mario","body":"faccio tardi"}'
      - text: "chiedi a gail se viene stasera"
        args: '{"recipient":"gail","body":"vieni stasera?"}'
      - text: "informa gail che l incontro è rimandato"
        args: '{"recipient":"gail","body":"l''incontro è rimandato"}'
      - text: "manda due righe a gail che sto arrivando"
        args: '{"recipient":"gail","body":"sto arrivando"}'
      - text: "gira un messaggio a mario che sono fuori"
        args: '{"recipient":"mario","body":"sono fuori"}'
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Messaggio

Manda un messaggio a qualcuno, oppure te lo prepara pronto da inviare.

## Frasi riconosciute

```
scrivi a mario che arrivo tardi
scrivi alla mamma che torno presto
manda un messaggio a gail dicendo che sono in ritardo
di' a mario che esco ora
avvisa gail che faccio tardi
fai sapere a gail che sono sull'autobus
manda un sms a gail ci vediamo alle 8
manda una mail a gail ricordati del latte
scrivi a mario buon compleanno su WhatsApp
rispondi a gail arrivo
```

Il servizio si indica con **su**, **via**, **tramite** o **con** — `… su
WhatsApp`. Se non lo dici, Ari usa quello impostato in **Manda i messaggi
con**, che di default è Messaggi.

Ometti il testo — `scrivi a gail` — e Ari ti chiede cosa vuoi dire.

## La preposizione conta

`scrivi`, `manda un messaggio`, `di'` e `fai sapere` vogliono la
preposizione davanti al nome: `scrivi **a** Mario`, `scrivi **alla**
mamma`, `scrivi **all'**avvocato. Senza, la skill non risponde — ed è
voluto: è l'unica cosa che distingue "scrivi a Gail" da "scrivi una
poesia", e senza quel vincolo la skill si prenderebbe mezze domande
rivolte ad Ari.

`avvisa`, `avverti` e `contatta` fanno eccezione: reggono l'oggetto
diretto (`avvisa Gail`) e sono già abbastanza specifici da soli.

## Inviare o preparare

Solo alcuni servizi possono essere inviati senza che tu tocchi il
telefono. Gli altri non permettono a un'altra app di inviare per tuo
conto, quindi Ari li apre con il messaggio già scritto e sei tu a
premere invia.

**Quando è Ari a inviare**, prima ti rilegge il messaggio e aspetta un
sì. Un messaggio alla persona sbagliata non si può richiamare, quindi
questo comportamento è attivo di default; lo disattivi da **Prima di
inviare**.

**Quando Ari lo prepara**, non chiede niente — stai per guardare il
messaggio e premere invia, e quel tocco è la conferma.

## Matrix

Matrix è l'unico servizio che Ari gestisce completamente da solo: non si
apre nessun'altra app, non c'è niente da premere. Metti il tuo server e
un token di accesso nelle impostazioni della skill, e Ari trova la
persona nella directory utenti del tuo server.

Due cose che non fa, di proposito:

- **Stanze cifrate.** La maggior parte dei DM Matrix è cifrata
  end-to-end, e Ari non ha modo di scriverci dentro. Te lo dice, invece
  di mandare qualcosa che il client del destinatario segnala come
  sospetto.
- **Scegliere fra due persone.** Se la directory restituisce più di una
  corrispondenza, Ari chiede un nome più completo invece di tirare a
  indovinare.

## Cosa non fa

**Rispondere a un messaggio appena ricevuto** non è questa skill. Serve
la notifica della conversazione stessa ed è una cosa a parte.

**Discord** non è supportato. Le identità Discord non hanno alcun legame
con la tua rubrica, quindi non c'è modo di capire chi sia "Gail" lì.

## I tuoi dati

Il messaggio va all'app che hai scelto, sul tuo dispositivo. Ari non ne
tiene copia e non manda niente altrove.
