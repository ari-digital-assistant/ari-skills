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
    version: "0.4.4"
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
        # `qual è` (e-grave) survives engine normalisation with the
        # accent intact, so the regex needs `è` not `e` — the prior
        # `qual e` form silently failed to fire against real input.
        # `prossimo promemoria` is a strong enough anchor that any
        # surrounding shape ("dimmi il prossimo promemoria") also
        # routes correctly.
        - regex: "\\bprossimo promemoria\\b"
          weight: 0.95
        - regex: "\\bche promemoria (ho|ho per)\\b"
          weight: 0.9
        - regex: "\\bquali promemoria\\b"
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
      - text: "imposta un promemoria per {text} {when}"
        weight: 0.75
        args:
          title: "{text}"
          when: "{when}"
      - text: "crea un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "fammi un promemoria per {text} {when}"
        weight: 0.75
        args:
          title: "{text}"
          when: "{when}"
      - text: "puoi impostarmi un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "vorrei un promemoria per {text} {when}"
        weight: 0.75
        args:
          title: "{text}"
          when: "{when}"
      - text: "mi serve un promemoria per {text} {when}"
        weight: 0.75
        args:
          title: "{text}"
          when: "{when}"
      - text: "ho bisogno di un promemoria per {text} {when}"
        weight: 0.75
        args:
          title: "{text}"
          when: "{when}"
      - text: "potresti creare un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "mi prepari un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "segnami un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "programma un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "metti in agenda un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "lasciami un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "non farmi dimenticare di {text}: crea un promemoria {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "assicurati che riceva un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "ricordamelo con un promemoria: {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "avvisami con un promemoria di {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "mandami un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "fai comparire un promemoria per {text} {when}"
        weight: 0.85
        args:
          title: "{text}"
          when: "{when}"
      - text: "sarebbe utile avere un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "posso avere un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "mi imposti un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "mi crei un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "tienimi da parte un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "annota un promemoria per {text} {when}"
        weight: 0.95
        args:
          title: "{text}"
          when: "{when}"
      - text: "ricordami di {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "imposta un promemoria per {text}"
        weight: 0.75
        args:
          title: "{text}"
      - text: "crea un promemoria senza orario per {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "segnami tra i promemoria di {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "aggiungi un promemoria per {text}"
        weight: 0.75
        args:
          title: "{text}"
      - text: "puoi lasciarmi un promemoria per {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "vorrei ricordarmi di {text}, aggiungi un promemoria"
        weight: 0.85
        args:
          title: "{text}"
      - text: "mi serve un promemoria senza scadenza per {text}"
        weight: 0.85
        args:
          title: "{text}"
      - text: "metti {text} nei miei promemoria"
        weight: 0.95
        args:
          title: "{text}"
      - text: "annota tra i promemoria di {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "fammi trovare un promemoria per {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "non devo dimenticare di {text}, crea un promemoria"
        weight: 0.95
        args:
          title: "{text}"
      - text: "puoi creare un promemoria non programmato per {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "salva come promemoria: {text}"
        weight: 0.95
        args:
          title: "{text}"
      - text: "tienimi a mente {text} con un promemoria"
        weight: 0.95
        args:
          title: "{text}"
      - text: "aggiungi {item} alla {list}"
        weight: 0.6
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "metti {item} nella {list}"
        weight: 0.6
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "segna {item} sulla {list}"
        weight: 0.95
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "puoi aggiungere {item} alla {list}"
        weight: 0.95
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "mi aggiungi {item} alla {list}"
        weight: 0.6
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "vorrei {item} nella {list}"
        weight: 0.6
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "fammi trovare {item} sulla {list}"
        weight: 0.95
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "annota {item} nella {list}"
        weight: 0.95
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "inserisci {item} nella {list}"
        weight: 0.95
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "puoi mettere {item} sulla {list}"
        weight: 0.6
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "non farmi dimenticare {item}, aggiungilo alla {list}"
        weight: 0.95
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "mi serve {item} nella {list}"
        weight: 0.6
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "aggiungimi {item} alla {list}"
        weight: 0.95
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "scrivi {item} sulla {list}"
        weight: 0.95
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "ricordati di mettere {item} nella {list}"
        weight: 0.95
        args:
          title: "{item}"
          list_hint: "{list}"
      - text: "ricordami di {title} {when}"
        weight: 0.95
        args:
          title: "{title}"
          when: "{when}"
      - text: "ricordami {when} di {title}"
        weight: 0.95
        args:
          title: "{title}"
          when: "{when}"
      - text: "imposta un promemoria di {title} {when}"
        weight: 0.75
        args:
          title: "{title}"
          when: "{when}"
      - text: "aggiungi {title} alla lista della {list_hint}"
        weight: 0.6
        args:
          title: "{title}"
          list_hint: "{list_hint}"
      - text: "metti {title} sulla lista della {list_hint}"
        weight: 0.6
        args:
          title: "{title}"
          list_hint: "{list_hint}"
      - text: "aggiungi {title} alla mia lista {list_hint}"
        weight: 0.75
        args:
          title: "{title}"
          list_hint: "{list_hint}"
      - text: "ricordami del{title} {when}"
        weight: 0.95
        args:
          title: "{title}"
          when: "{when}"
      - text: "avvisami {when} di {title}"
        weight: 0.95
        args:
          title: "{title}"
          when: "{when}"
      - text: "dimmi {when} di {title}"
        weight: 0.6
        args:
          title: "{title}"
          when: "{when}"
      - text: "fammi sapere {when} di {title}"
        weight: 0.6
        args:
          title: "{title}"
          when: "{when}"
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

Creare un promemoria senza orario non è mai un'analisi ad alta
affidabilità (dalla v0.4.0): passa dal giro di consultazione
dell'assistente, quindi si ottiene una domanda di conferma o una scheda
annullabile, invece di un'attività senza orario registrata in silenzio.
Il riconoscimento vocale che tronca un "tra un'ora" finale faceva
esattamente questo. Le aggiunte alle liste personalizzate non sono
interessate — per loro l'assenza di orario è la norma.

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
