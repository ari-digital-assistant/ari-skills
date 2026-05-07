---
# `name` must match the directory (`reminder/`) — it's the stable
# system identifier, not a display field. Per-locale display strings
# live in `description` (below) and the markdown body. Don't translate
# this.
name: reminder
description: Imposta promemoria con orario e voci di lista senza orario. Indirizza all'app delle attività dell'utente (predefinita), al calendario, o a entrambi, con liste personalizzate vocalmente come "aggiungi latte alla lista della spesa".
license: MIT
metadata:
  ari:
    id: dev.heyari.reminder
    version: "0.2.0"
    author: Ari core team
    homepage: https://github.com/ari-digital-assistant/ari-skills
    engine: ">=0.3"
    capabilities: [calendar, tasks]
    languages: [en, it]
    specificity: high
    matching:
      patterns:
        # Italian "ricordami" = "remind me". Both with and without the
        # connector "di" — voice users skip it as often as English users
        # skip "to". Patterns assume input has been through
        # `normalize_input` (lowercased, apostrophes/punctuation stripped).
        - regex: "\\bricordami\\b"
          weight: 0.95
        - regex: "\\b(imposta|crea) un promemoria\\b"
          weight: 0.95
        # Named-list patterns: "aggiungi X alla lista della spesa",
        # "metti X sulla lista". The connector words ("alla"/"sulla"/"nella"
        # and their plural forms) are articulated prepositions
        # specific to Italian — they fold the article into the
        # preposition. The "lista" suffix is optional in colloquial
        # speech ("aggiungi latte alla spesa" is acceptable).
        - regex: "\\b(aggiungi|metti) .+ (alla|sulla|nella|alle|sulle|nelle) (lista|spesa)\\b"
          weight: 0.95
        - regex: "\\b(aggiungi|metti) .+ (alla|sulla|nella) \\w+\\b"
          weight: 0.9
        # Read-only queries — Italian forms of "what reminders do I
        # have today/tomorrow", "what's my next reminder".
        - regex: "\\bqual e (il|mio) prossimo promemoria\\b"
          weight: 0.95
        - regex: "\\bche promemoria (ho|ho per)\\b"
          weight: 0.9
        - regex: "\\bho (qualche|dei) promemoria\\b"
          weight: 0.9
        - regex: "\\bcosa ho (oggi|domani|in programma)\\b"
          weight: 0.85
        # Internal cancel/confirm round-trips — same magic-prefix tokens
        # as SKILL.en.md. These are alphanumeric-safe and survive
        # normalisation regardless of locale; they exist to round-trip
        # card actions back to the skill.
        - regex: "^aricancelreminder\\b"
          weight: 1.0
        - regex: "^ariconfirmreminder\\b"
          weight: 1.0
      custom_score: false
    examples:
      - text: "ricordami di portare fuori il cane alle 17"
        args:
          title: "portare fuori il cane"
          when: "alle 17"
      - text: "ricordami di comprare il latte"
        args:
          title: "comprare il latte"
      - text: "ricordami di portare fuori la spazzatura stasera"
        args:
          title: "portare fuori la spazzatura"
          when: "stasera"
      - text: "ricordami alle 9 domani di chiamare il dentista"
        args:
          title: "chiamare il dentista"
          when: "alle 9 domani"
      - text: "ricordami tra 30 minuti di controllare il forno"
        args:
          title: "controllare il forno"
          when: "tra 30 minuti"
      - text: "imposta un promemoria di mandare email a sara venerdì alle 15"
        args:
          title: "mandare email a sara"
          when: "venerdì alle 15"
      - text: "aggiungi latte alla lista della spesa"
        args:
          title: "latte"
          list_hint: "spesa"
      - text: "metti uova sulla lista della spesa"
        args:
          title: "uova"
          list_hint: "spesa"
      - text: "aggiungi revisione scadenze alla mia lista lavoro"
        args:
          title: "revisione scadenze"
          list_hint: "lavoro"
      - text: "ricordami della riunione alle 16"
        args:
          title: "la riunione"
          when: "alle 16"
      # Paraphrases without literal "ricordami" / "imposta un
      # promemoria" / "aggiungi alla lista" triggers — same routing-
      # teaching rationale as SKILL.en.md.
      - text: "avvisami alle 17 di portare fuori il cane"
        args:
          title: "portare fuori il cane"
          when: "alle 17"
      - text: "dimmi alle 9 domani di chiamare il dentista"
        args:
          title: "chiamare il dentista"
          when: "alle 9 domani"
      - text: "svegliami alle 7"
        args:
          title: "svegliarsi"
          when: "alle 7"
      - text: "fammi sapere stasera di portare fuori la spazzatura"
        args:
          title: "portare fuori la spazzatura"
          when: "stasera"
    settings:
      - key: destination
        label: Salva i promemoria in
        type: select
        default: tasks
        options:
          - value: tasks
            label: Attività
          - value: calendar
            label: Calendario
          - value: both
            label: Entrambi
      - key: default_calendar
        label: Calendario predefinito
        type: device_calendar
        show_when:
          key: destination
          equals: [calendar, both]
      - key: default_task_list
        label: Lista attività predefinita
        type: device_task_list
        show_when:
          key: destination
          equals: [tasks, both]
    wasm:
      module: skill.wasm
      memory_limit_mb: 4
---

# Promemoria

Imposta promemoria con orario e voci di lista senza orario, indirizzandoli
all'app delle attività dell'utente, al calendario, o a entrambi in base
all'impostazione **Salva i promemoria in**.

## Frasi supportate

Destinazione predefinita (usa la lista / calendario predefinito selezionato):

- `ricordami di portare fuori il cane alle 17` — con orario
- `ricordami di comprare il latte` — senza orario (va sempre in Attività)
- `ricordami alle 9 domani di chiamare il dentista` — data relativa + orario esplicito
- `ricordami tra 30 minuti di controllare il forno` — orario relativo
- `imposta un promemoria di mandare email a sara venerdì alle 15` — giorno della settimana esplicito

Lista personalizzata (sostituisce la lista predefinita — la voce vince sempre):

- `aggiungi latte alla lista della spesa` — lista personalizzata, senza orario
- `metti uova sulla lista della spesa` — stessa forma, verbo "metti"
- `aggiungi revisione scadenze alla mia lista lavoro` — qualsiasi lista personalizzata

Se non viene fornito un orario, il promemoria viene creato come attività
senza orario. Se viene fornito un orario, viene emesso come timestamp
ISO-8601 assoluto; il frontend si occupa di scriverlo come VTODO con
data di scadenza e/o come VEVENT con un avviso a seconda dell'impostazione
di destinazione.

## Impostazioni

- **Salva i promemoria in** — Attività (predefinito), Calendario, o Entrambi.
  Attività è disabilitato se non è installata nessuna app compatibile con
  OpenTasks (Tasks.org, jtx Board, OpenTasks, ecc.); il pannello delle
  impostazioni mostra i link per l'installazione in quel caso.
- **Calendario predefinito** — scelto da `CalendarContract.Calendars`.
- **Lista attività predefinita** — scelta dal ContentProvider OpenTasks.

## Note

L'analisi temporale supporta sia l'inglese ("at 5pm", "tomorrow", "in 30
minutes") che l'italiano ("alle 17", "domani", "tra 30 minuti") nello
stesso parser. Le risposte vocali e le etichette delle schede vengono
caricate da `strings/it.json`. Per aggiungere una terza lingua: estendere
i dizionari del parser con i token della nuova lingua, aggiungere un
nuovo `SKILL.<locale>.md` e un nuovo `strings/<locale>.json`.

I promemoria senza orario vengono sempre indirizzati ad Attività
indipendentemente dall'impostazione **Salva i promemoria in**, poiché
le griglie del calendario non hanno una rappresentazione utile per un
evento senza orario.
