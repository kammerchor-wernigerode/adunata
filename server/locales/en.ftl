beds-needed =
    { $beds ->
        [one] One bed needed.
       *[other] { $beds } beds needed.
    }

attendance =
    { $attendance ->
        [accepted] accepted
        [tentative] tentatively accepted
       *[declined] declined
    }

response-mail-subject = Response from { $name }
response-mail-body = { $name } { attendance } the invitation to the gathering from { $span }.

date-span-same-month = { $month } { $startDay }–{ $endDay }, { $year }

month-1 = January
month-2 = February
month-3 = March
month-4 = April
month-5 = May
month-6 = June
month-7 = July
month-8 = August
month-9 = September
month-10 = October
month-11 = November
month-12 = December
