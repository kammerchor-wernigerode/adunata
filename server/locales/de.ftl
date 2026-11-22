date-span-same-year = { $startDay }. { $startMonth } – { $endDay }. { $endMonth } { $year }
date-span-across-years = { $startDay }. { $startMonth } { $startYear } – { $endDay }. { $endMonth } { $endYear }
date-single-day = { $day }. { $month } { $year }
date-span-same-month = { $startDay }.–{ $endDay }. { $month } { $year }

month-1 = Januar
month-2 = Februar
month-3 = März
month-4 = April
month-5 = Mai
month-6 = Juni
month-7 = Juli
month-8 = August
month-9 = September
month-10 = Oktober
month-11 = November
month-12 = Dezember

product-name = Adunata
privacy-notice = Datenschutzerklärung
imprint = Impressum

status-pending = Offen
status-accepted = Zugesagt
status-tentative = Vorläufig zugesagt
status-declined = Abgesagt

invitation-mail-subject = Einladung: { $title }, { $span }
invitation-mail-greeting = Hallo { $firstName },
invitation-mail-details = du bist zu { $title } in { $location } eingeladen, { $span }.
invitation-mail-answer = Sag uns über deinen persönlichen Link, ob und wann du kommst. Bitte leite ihn nicht weiter, denn er funktioniert ohne Passwort.

cancellation-mail-subject = Abgesagt: { $title }, { $span }
cancellation-mail-body = { $title } in { $location }, { $span }, findet nicht statt.

sign-in-mail-subject = Dein Anmeldelink für Adunata
sign-in-mail-body = Hallo { $firstName }, öffne diesen Link innerhalb einer Stunde, um dich bei Adunata anzumelden. Er funktioniert einmal.
sign-in-mail-ignore = Wenn du keine Anmeldung angefordert hast, ignoriere diese Mail.

nav-label = Hauptnavigation
nav-members = Mitglieder
nav-sign-out = Abmelden
account-menu-label = Konto von { $name }
account-own-entry = Mein Mitgliedseintrag

sign-in-title = Anmelden
sign-in-mail = Mailadresse
sign-in-send = Anmeldelink senden
sign-in-sent = Wenn diese Adresse einem Organisator gehört, ist ein Anmeldelink unterwegs. Er funktioniert einmal innerhalb einer Stunde.
sign-in-confirm-title = Anmeldung bestätigen
sign-in-confirm-text = Drücke den Knopf, um dich auf diesem Gerät anzumelden.
sign-in-confirm = Anmelden
sign-in-invalid = Dieser Link wurde schon benutzt oder ist abgelaufen.
sign-in-request-new = Neuen Link anfordern

nav-gatherings = Treffen
gatherings-title = Treffen
gatherings-create = Treffen anlegen
gatherings-upcoming = Kommende
gatherings-past = Vergangene
gatherings-none = Noch keine.
gathering-new-title = Treffen anlegen
gathering-edit-title = { $title } bearbeiten
gathering-edit = Treffen bearbeiten
gathering-title = Titel
gathering-location = Ort
gathering-description = Beschreibung
gathering-first-day = Erster Tag
gathering-last-day = Letzter Tag
gathering-first-day-months = Monate für den ersten Tag
gathering-last-day-months = Monate für den letzten Tag
date-picker-choose-day = Tag wählen
gathering-save = Speichern
gathering-cancelled = Abgesagt
error-last-day-before-first-day = Der letzte Tag kann nicht vor dem ersten liegen.
error-gathering-ended = Dieses Treffen ist vorbei und lässt sich nicht mehr ändern.

invitations-title = Einladungen
invitations-invite-all = { $count ->
        [one] Das eine noch nicht eingeladene aktive Mitglied einladen
       *[other] Alle { $count } noch nicht eingeladenen aktiven Mitglieder einladen
    }
invitations-choose = Mitglieder zum Einladen auswählen
invitations-invite-chosen = { $count ->
        [0] Ausgewählte einladen
        [one] Ein ausgewähltes Mitglied einladen
       *[other] { $count } ausgewählte Mitglieder einladen
    }
invitations-everyone-invited = Alle aktiven Mitglieder sind eingeladen.
invitations-sent = { $count ->
        [0] Niemand Neues einzuladen.
        [one] Eine Einladung verschickt.
       *[other] { $count } Einladungen verschickt.
    }
invitation-link-active = Link gültig
invitation-link-revoked = Link widerrufen
invitation-mail-waiting = Mail wartet
invitation-mail-sent = Mail verschickt
invitation-mail-failed = Mail gescheitert
invitation-resend = Neu senden
invitation-revoke = Link widerrufen
invitation-resent = Neuer Link an { $name } verschickt.
invitation-revoked = Der Link von { $name } funktioniert nicht mehr.
gathering-cancel = Treffen absagen
gathering-cancel-confirm = { $title } absagen und alle Eingeladenen per Mail informieren?
gathering-cancel-yes = Treffen absagen

response-days = Tage des Treffens
invitation-invalid-title = Link ungültig
invitation-invalid = Dieser Link ist nicht gültig. Vielleicht wurde er durch einen neueren ersetzt. Bitte deine Organisatoren um einen neuen Link.
invitation-state-begun = Das Treffen hat begonnen. Deine Rückmeldung lässt sich hier nicht mehr ändern.
invitation-state-ended = Das Treffen ist vorbei.
invitation-state-cancelled = Dieses Treffen ist abgesagt.

overview-title = Rückmeldungen
overview-total-accepted = Zusagen
overview-total-tentative = Vorläufig
overview-total-declined = Absagen
overview-total-pending = Offen
overview-beds-needed = Betten gesucht
overview-beds-offered = Betten angeboten
overview-bed-even = so viele wie gesucht
overview-bed-surplus = { $count ->
        [one] eins mehr als gesucht
       *[other] { $count } mehr als gesucht
    }
overview-bed-shortage = { $count ->
        [one] eins weniger als gesucht
       *[other] { $count } weniger als gesucht
    }
overview-filter = Nach Status filtern
overview-filter-all = Alle { $count }
overview-filter-accepted = Zusagen
overview-filter-tentative = Vorläufig
overview-filter-declined = Absagen
overview-filter-pending = Offen
overview-column-name = Name
overview-column-status = Status
overview-column-period = Zeitraum
overview-column-beds = Betten
overview-column-comment = Kommentar
overview-column-actions = Aktionen
overview-beds-need = { $count ->
        [one] braucht eins
       *[other] braucht { $count }
    }
overview-beds-offer = { $count ->
        [one] bietet eins
       *[other] bietet { $count }
    }
overview-whole-gathering = ganzes Treffen
overview-period = { $arrival } bis { $departure }
overview-period-start = Beginn
overview-period-end = Ende
overview-outside-span = außerhalb der Tage

response-question = Kommst du, { $firstName }?
response-question-for = Kommt { $name }?
response-status-accepted = Ja
response-status-tentative = Vielleicht
response-status-declined = Nein
response-when = Wann bist du da?
response-arrival = Anreise
response-departure = Abreise
response-arrival-day = Tag der Anreise
response-departure-day = Tag der Abreise
response-arrival-months = Monate für die Anreise
response-departure-months = Monate für die Abreise
response-arrival-time = Uhrzeit der Anreise
response-departure-time = Uhrzeit der Abreise
response-when-hint = { $days ->
        [one] Ohne Uhrzeit planen wir dich für den ganzen Tag ein.
       *[other] Ohne Angabe planen wir dich für alle { $days } Tage ein.
    }
response-beds = Übernachtung
response-beds-need = Ich brauche ein Bett
response-beds-offer = Ich biete Betten an
response-beds-none = Nichts davon
response-bed-count = Wie viele?
response-comment = Kommentar
response-comment-hint = Nur Organisatoren lesen deinen Kommentar.
response-save = Antwort speichern
response-saved = Antwort gespeichert. Du kannst sie bis { $day } ändern.
response-saved-for = Antwort von { $name } gespeichert.
response-visibility = Andere Eingeladene sehen deinen Namen und ob du kommst.
response-yours = Deine Antwort
response-error-begun = Das Treffen hat begonnen, deshalb lässt sich deine Antwort hier nicht mehr ändern. Wende dich an deine Organisatoren.
response-error-beds = Wähle, ob du Betten brauchst oder anbietest.
response-error-bed-count = Gib an, wie viele Betten, von 1 bis 20.
response-error-bed-count-too-large = Gib höchstens 20 Betten an.
response-error-comment-too-long = Der Kommentar darf höchstens 2000 Zeichen haben.
response-error-departure-before-arrival = Die Abreise kann nicht vor der Anreise liegen.
response-error-time = Wähle eine Uhrzeit innerhalb der Tage des Treffens in Schritten von 30 Minuten.
response-error-day-and-time = Wähle Tag und Uhrzeit.
attendees-title = { $tentative ->
        [0] Dabei sind { $accepted }
       *[other] Dabei sind { $accepted }, vielleicht { $tentative }
    }
attendees-more = und { $count } weitere
attendees-others = Abgesagt: { $declined }. Noch keine Antwort: { $pending }.
invitation-page-title = Einladung von { $name }
invitation-changed-by-member = vom Mitglied geändert am { $at }
invitation-changed-by-organizer = von Organisatoren geändert am { $at }
invitation-link-title = Persönlicher Link

members-title = Mitglieder
members-add = Mitglied hinzufügen
members-search = Nach Name oder Mailadresse suchen
members-filter = Anzeigen
members-filter-all = Alle
members-filter-active = Aktive
members-filter-inactive = Inaktive
members-empty = Kein Mitglied passt.
members-copy-mail = Mailadresse kopieren
members-mail-copied = Mailadresse kopiert
members-copy-failed = Die Mailadresse ließ sich nicht kopieren. Markiere und kopiere sie von Hand.
members-count = { $count ->
        [one] Ein Mitglied
       *[other] { $count } Mitglieder
    }

member-first-name = Vorname
member-last-name = Nachname
member-name = Name
member-mail = Mailadresse
member-language = Sprache der Mails
member-status = Status
member-active = Aktiv
member-inactive = Inaktiv
member-organizer = Organisator
member-role = Rolle
member-organizer-hint = Organisatoren melden sich an und verwalten Mitglieder und Treffen.
member-new-title = Mitglied hinzufügen
member-edit-title = { $name } bearbeiten
member-save = Speichern
member-saved = Gespeichert.
member-delete = Mitglied löschen
member-delete-confirm = { $name } mit allen Einladungen, Rückmeldungen und wartenden Mails löschen?
member-delete-yes = Löschen
member-cancel = Abbrechen

error-field-required = Fülle dieses Feld aus.
error-field-too-long = Das ist zu lang.
error-mail-invalid = Gib eine Mailadresse wie name@example.org ein.
error-mail-taken = Die Adresse { $conflict } ist schon vergeben.
error-last-organizer = Die Instanz braucht mindestens einen aktiven Organisator.
error-organizer-inactive = Nur aktive Mitglieder können Organisatoren sein.
error-unexpected = Etwas ist schiefgegangen. Versuch es noch einmal.
error-page-title = Seite nicht verfügbar
error-retry = Erneut versuchen

language-name-de = Deutsch
language-name-en = English

language-label = Sprache


error-required = Gib einen Namen ein.
error-too-long = Der Name ist zu lang.
error-too-large = Die Zahl ist zu groß.
error-taken = Dieser Name ist schon vergeben.
error-number = Gib eine ganze Zahl ein.
error-date = Gib ein gültiges Datum ein.
error-invalid = Dieser Wert ist ungültig.
