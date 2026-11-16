beds-needed =
    { $beds ->
        [one] Ein Bett benötigt.
       *[other] { $beds } Betten benötigt.
    }

attendance =
    { $attendance ->
        [accepted] zugesagt
        [tentative] vorläufig zugesagt
       *[declined] abgesagt
    }

response-mail-subject = Rückmeldung von { $name }
response-mail-body = { $name } hat die Einladung zur Zusammenkunft vom { $span } { attendance }.

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

sign-in-mail-subject = Dein Anmeldelink für Adunata
sign-in-mail-body = Hallo { $firstName }, öffne diesen Link innerhalb einer Stunde, um dich bei Adunata anzumelden. Er funktioniert einmal.
sign-in-mail-ignore = Wenn du keine Anmeldung angefordert hast, ignoriere diese Mail.

nav-label = Hauptnavigation
nav-members = Mitglieder
nav-sign-out = Abmelden

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
gathering-save = Speichern
gathering-cancelled = Abgesagt
error-last-day-before-first-day = Der letzte Tag kann nicht vor dem ersten liegen.
error-gathering-ended = Dieses Treffen ist vorbei und lässt sich nicht mehr ändern.

members-title = Mitglieder
members-add = Mitglied hinzufügen
members-search = Nach Name oder Mailadresse suchen
members-filter = Anzeigen
members-filter-all = Alle
members-filter-active = Aktive
members-filter-inactive = Inaktive
members-empty = Kein Mitglied passt.
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

language-name-de = Deutsch
language-name-en = English

language-label = Sprache

spike-title = Rückmeldung
spike-name = Name
spike-attendance = Teilnahme
spike-attendance-accepted = Zugesagt
spike-attendance-tentative = Vorläufig
spike-attendance-declined = Abgesagt
spike-arrival = Anreise
spike-beds = Betten
spike-save = Speichern
spike-saved = Rückmeldung von { $name } gespeichert.

error-required = Gib einen Namen ein.
error-too-long = Der Name ist zu lang.
error-too-large = Die Zahl ist zu groß.
error-taken = Dieser Name ist schon vergeben.
error-number = Gib eine ganze Zahl ein.
error-date = Gib ein gültiges Datum ein.
error-invalid = Dieser Wert ist ungültig.
