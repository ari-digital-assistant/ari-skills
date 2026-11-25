# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- chatgpt 0.5.0, claude 0.5.0 and gemini 0.5.0 search the web for answers that need
  current information (weather, news, prices, scores) on engines that support
  `web_search`: ChatGPT through OpenAI's Responses API, Claude with Anthropic's web
  search tool (up to three searches a question), Gemini grounded in Google Search
  through its own API. Their instructions for those calls no longer say they have no
  live data. ChatGPT and Gemini allow 2,048 tokens, since reasoning counts against the
  limit. Older engines ignore the new block and answer as before.
- Suggest phrases: alarm 0.1.3, coin-flip 0.1.1, home-assistant 0.3.3, message 0.2.5,
  music 0.2.2, navigation 0.1.3, reminder 0.5.1, timer 0.2.3 and weather 0.2.3 list, in
  English and Italian, the phrases that clearly need them (`metadata.ari.suggest`), so
  Ari can offer them to someone who hasn't installed them. `index.json` carries the
  lists per language as `suggest`
  ([ari-android#44](https://github.com/ari-digital-assistant/ari-android/issues/44)).
- navigation 0.1.2: a Home address setting. "Take me home" and "portami a casa" go
  there in any maps app; left blank, turn-by-turn asks Google Maps for the home saved
  in the user's account.
- SDK: `Navigate::saved_home()`, which sets the navigate action's new `place: "home"`.

### Fixed

- reminder 0.5.2: its cards' styles are lowercase, as the envelope format has them, so
  the "Is this right?" card's Yes is the primary button, the partial card's Cancel is
  destructive and a low-confidence card is a warning. Written in capitals, they all fell
  back to the default.
- timer 0.2.4: a timer's Cancel button, on its card or its notification, cancels that
  timer. It used to say "cancel my timer", so with two unnamed timers running Ari asked
  which one and cancelled neither, though the card went. The buttons now send
  `aricanceltimer <id>`; a timer that has already finished gets "That timer has already
  finished.", or "Stopped your timer." while it is still ringing.
- `models.json`: ChatGPT's Fast and Balanced models (gpt-6-luna, gpt-6.1-terra) leave
  out `temperature`. models.dev says they take it, but the API rejects anything but the
  default with a 400, so every routing call and skill consultation through ChatGPT on
  those tiers failed. `build-models.sh` now omits sampling parameters for every OpenAI
  reasoning model, whatever models.dev says.
- music 0.2.1: Italian phrasings the manifest already listed now work. "Metti su Vasco
  Rossi" searched for "su vasco rossi"; "voglio ascoltare", "fammi sentire", "mettimi"
  and "puoi mettere" asked what to play; "più forte" never matched because the engine
  keeps the accent; "cambia canzone", "basta musica" and "torna alla canzone di prima"
  weren't recognised.
- timer 0.2.2: Italian works. The parser only knew English, so every Italian request
  ("imposta un timer per 5 minuti") reached the skill and got "non ho capito". It now
  reads Italian verbs, units, number words ("cinque", "ventitré"), "un quarto d'ora",
  "mezz'ora" and "un'ora e mezza", and names after the noun ("il timer pasta"), which the
  card's Italian cancel button relies on
  ([ari-android#35](https://github.com/ari-digital-assistant/ari-android/issues/35)).
- timer 0.2.2: the confirmation no longer capitalises mid-sentence ("Set A timer for 5
  minutes").
- navigation 0.1.1: the phrasings the manifests list now find the destination. 36 of
  the English examples and 41 of the Italian ones answered "Where would you like to
  go?", including "I want to go to Rome", "navigate home", "voglio andare a casa",
  "naviga verso Milano" and "accompagnami in ufficio".
- navigation 0.1.2: "start navigation to…" and "avvia il navigatore…" reach navigation
  instead of opening an app called "navigation to…".
