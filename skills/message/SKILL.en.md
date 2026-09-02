---
name: message
description: >
  Send a message to someone through SMS, WhatsApp, Telegram, Signal,
  Messenger, Slack, Matrix or email. Say who and what, and Ari either
  sends it or opens the app with your message ready to go.
metadata:
  ari:
    id: dev.heyari.message
    version: "0.2.2"
    type: skill
    author: Ari Project
    homepage: https://github.com/ari-digital-assistant/ari-skills
    license: MIT
    engine: ">=0.3"
    languages: [en, it]
    capabilities: [send_message, contacts, http, reply]
    specificity: high
    matching:
      # custom_score is on, so the engine calls the module's `score` export
      # and never runs these patterns. They're kept accurate because they
      # document the shapes the parser accepts, and the validator requires
      # at least one entry.
      #
      # Why custom scoring: "tell" needs a negative match. "tell me a joke"
      # and "tell me the time" must not land here, and Rust's regex crate
      # has no lookaround, so `\btell (?!me\b)` cannot be expressed.
      custom_score: true
      patterns:
        - regex: "\\b(send|message|text|tell|whatsapp|telegram|signal|slack)\\b"
          weight: 0.9
        - regex: "\\blet \\w+ know\\b"
          weight: 0.9
    settings:
      - key: confirm_before_sending
        label: Before sending
        type: select
        default: always
        options:
          - value: always
            label: Read it back and ask
          - value: never
            label: Send straight away
      - key: matrix_homeserver
        label: Matrix server
        type: text
        keyboard: url
      - key: matrix_token
        label: Matrix access token
        type: secret
      - key: default_service
        label: Send messages with
        type: select
        default: sms
        options:
          - value: sms
            label: Messages
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
    examples:
      - text: "start a WhatsApp message to {recipient}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          service: "whatsapp"
      - text: "tell {recipient} {body}"
        weight: 0.6
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "tell {recipient} that {body}"
        weight: 0.6
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "please tell {recipient} {body}"
        weight: 0.6
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "could you tell {recipient} that {body}"
        weight: 0.6
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "would you mind telling {recipient} that {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "let {recipient} know {body}"
        weight: 0.6
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "please let {recipient} know that {body}"
        weight: 0.6
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "can you let {recipient} know {body}"
        weight: 0.6
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "could you let {recipient} know that {body}"
        weight: 0.6
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "make sure {recipient} gets my message that {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "pass this message to {recipient}: {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "pass along a message to {recipient} saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "get a message to {recipient} saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "relay this message to {recipient}: {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "send {recipient} a message saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "send a message to {recipient} saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "send {recipient} this message: {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "please send {recipient} a message saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "can you send {recipient} a message saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "could you send a message to {recipient} saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "would you send {recipient} a message saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "i need you to send {recipient} a message saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "i want to send {recipient} a message saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "i'd like to send a message to {recipient} saying {body}"
        weight: 0.85
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "help me send {recipient} a message saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "message {recipient} {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "message {recipient}"
        weight: 0.95
        args:
          recipient: "{recipient}"
      - text: "can you message {recipient} saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "could you message {recipient} and say {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "i need to message {recipient} that {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
      - text: "i want to message {recipient}"
        weight: 0.95
        args:
          recipient: "{recipient}"
      - text: "text {recipient} {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "sms"
      - text: "send a text to {recipient} saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "sms"
      - text: "please text {recipient} {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "sms"
      - text: "can you text {recipient}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          service: "sms"
      - text: "i need to text {recipient} that {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "sms"
      - text: "whatsapp {recipient} {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "whatsapp"
      - text: "send {recipient} a WhatsApp message saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "whatsapp"
      - text: "telegram {recipient} {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "telegram"
      - text: "send {recipient} a Telegram message saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "telegram"
      - text: "send {recipient} a Signal message saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "signal"
      - text: "message {recipient} via Signal and say {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "signal"
      - text: "send {recipient} a Messenger message saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "messenger"
      - text: "send {recipient} a Slack message saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "slack"
      - text: "send {recipient} a Matrix message saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "matrix"
      - text: "email {recipient} {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "email"
      - text: "send an email to {recipient} saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "email"
      - text: "please email {recipient} and say {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "email"
      - text: "can you email {recipient}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          service: "email"
      - text: "send {recipient} a message via {service} saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "{service}"
      - text: "please send a message to {recipient} via {service} saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "{service}"
      - text: "can you send {recipient} a message using {service} saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "{service}"
      - text: "message {recipient} via {service} and say {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "{service}"
      - text: "i need to get a message to {recipient} via {service} saying {body}"
        weight: 0.95
        args:
          recipient: "{recipient}"
          body: "{body}"
          service: "{service}"
      - text: "tell mario i will be home soon"
        weight: 0.6
      - text: "send a message to gail saying i am running late"
        weight: 0.95
      - text: "message sam on my way"
        weight: 0.95
      - text: "text gail see you at 8"
        weight: 0.95
      - text: "whatsapp mario happy birthday"
        weight: 0.75
      - text: "let gail know i am on the bus"
        weight: 0.6
      - text: "send an email to gail remember the milk"
        weight: 0.85
      - text: "tell mario i am outside on telegram"
        weight: 0.95
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Message

Sends a message to someone, or gets one ready for you to send.

## Supported utterances

```
tell mario i'll be home soon
message gail on my way
send a message to gail saying i'm running late
send gail a message saying i'm running late
text gail see you at 8
let gail know i'm on the bus
whatsapp mario happy birthday
tell mario i'm outside on telegram
send an email to gail remember the milk
email gail the invoice is attached
```

Name the service with **on**, **via** or **over** — `… on WhatsApp` — or use
it as the verb: `whatsapp mario …`, `text gail …`. Say nothing and Ari uses
the service from **Send messages with**, which defaults to Messages.

Leave the message off — `message gail` — and Ari asks what you want to say.

## Sending versus preparing

Only some services can be sent without you touching the phone. The rest have
no way for another app to send on your behalf, so Ari opens them with your
message already typed and you tap send.

**When Ari sends it itself**, it reads the message back first and waits for a
yes. A message to the wrong person cannot be recalled, so this is on by
default; turn it off under **Before sending**.

**When Ari prepares it**, there's no question — you're about to look at the
message and tap send, and that tap is the confirmation.

## Matrix

Matrix is the one service Ari sends entirely by itself — no other app opens,
nothing to tap. Fill in your server and an access token under the skill's
settings, and Ari finds the person through your server's own user directory.

Two things it won't do, deliberately:

- **Encrypted rooms.** Most Matrix DMs are end-to-end encrypted, and Ari has
  no way to write into one. It says so rather than sending something the
  recipient's client flags as suspect.
- **Guess between two people.** If the directory returns more than one match
  Ari asks for a fuller name instead of picking.

## What it doesn't do

**Replying to a message you just received** isn't this skill. That needs the
conversation's own notification and is a separate thing.

**Discord** isn't supported. Discord identities have no connection to your
address book, so there's no way to work out who "Gail" is there.

## Your data

The message goes to the app you chose, on your device. Ari doesn't keep a
copy, and nothing is sent anywhere else.
