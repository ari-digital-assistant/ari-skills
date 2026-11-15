# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- timer 0.2.2: Italian works. The parser only knew English, so every Italian request
  ("imposta un timer per 5 minuti") reached the skill and got "non ho capito". It now
  reads Italian verbs, units, number words ("cinque", "ventitré"), "un quarto d'ora",
  "mezz'ora" and "un'ora e mezza", and names after the noun ("il timer pasta"), which the
  card's Italian cancel button relies on
  ([ari-android#35](https://github.com/ari-digital-assistant/ari-android/issues/35)).
- timer 0.2.2: the confirmation no longer capitalises mid-sentence ("Set A timer for 5
  minutes").
