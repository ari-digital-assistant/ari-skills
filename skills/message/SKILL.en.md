---
name: message
description: >
  Send a message to someone through SMS, WhatsApp, Telegram, Signal,
  Messenger, Slack, Matrix or email. Say who and what, and Ari either
  sends it or opens the app with your message ready to go.
metadata:
  ari:
    id: dev.heyari.message
    version: "0.2.1"
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
      - text: "tell mario i will be home soon"
        args: '{"recipient":"mario","body":"i will be home soon"}'
      - text: "send a message to gail saying i am running late"
        args: '{"recipient":"gail","body":"i am running late"}'
      - text: "message sam on my way"
        args: '{"recipient":"sam","body":"on my way"}'
      - text: "text gail see you at 8"
        args: '{"recipient":"gail","body":"see you at 8","service":"sms"}'
      - text: "whatsapp mario happy birthday"
        args: '{"recipient":"mario","body":"happy birthday","service":"whatsapp"}'
      - text: "let gail know i am on the bus"
        args: '{"recipient":"gail","body":"i am on the bus"}'
      - text: "send an email to gail remember the milk"
        args: '{"recipient":"gail","body":"remember the milk","service":"email"}'
      - text: "tell mario i am outside on telegram"
        args: '{"recipient":"mario","body":"i am outside","service":"telegram"}'
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
