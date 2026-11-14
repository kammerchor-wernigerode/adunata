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

language-label = Sprache

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
