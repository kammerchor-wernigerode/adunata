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
date-span-same-year = { $startMonth } { $startDay } – { $endMonth } { $endDay }, { $year }
date-span-across-years = { $startMonth } { $startDay }, { $startYear } – { $endMonth } { $endDay }, { $endYear }
date-single-day = { $month } { $day }, { $year }

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

product-name = Adunata
privacy-notice = Privacy notice
imprint = Imprint

status-pending = Pending
status-accepted = Accepted
status-tentative = Tentative
status-declined = Declined

invitation-mail-subject = Invitation: { $title }
invitation-mail-greeting = Hello { $firstName },
invitation-mail-details = you are invited to { $title } in { $location }, { $span }.
invitation-mail-answer = Tell us through your personal link whether and when you come. Please do not forward it, because it works without a password.

cancellation-mail-subject = Cancelled: { $title }
cancellation-mail-body = { $title } in { $location }, { $span }, does not take place.

sign-in-mail-subject = Your sign-in link for Adunata
sign-in-mail-body = Hello { $firstName }, open this link within one hour to sign in to Adunata. It works once.
sign-in-mail-ignore = If you did not ask to sign in, ignore this mail.

nav-label = Main navigation
nav-members = Members
nav-sign-out = Sign out

sign-in-title = Sign in
sign-in-mail = Mail address
sign-in-send = Send sign-in link
sign-in-sent = If this address belongs to an organizer, a sign-in link is on its way. It works once within one hour.
sign-in-confirm-title = Confirm sign-in
sign-in-confirm-text = Press the button to sign in on this device.
sign-in-confirm = Sign in
sign-in-invalid = This link was already used or has expired.
sign-in-request-new = Request a new link

nav-gatherings = Gatherings
gatherings-title = Gatherings
gatherings-create = Create gathering
gatherings-upcoming = Upcoming
gatherings-past = Past
gatherings-none = None yet.
gathering-new-title = Create gathering
gathering-edit-title = Edit { $title }
gathering-edit = Edit gathering
gathering-title = Title
gathering-location = Location
gathering-description = Description
gathering-first-day = First day
gathering-last-day = Last day
gathering-save = Save
gathering-cancelled = Cancelled
error-last-day-before-first-day = The last day cannot lie before the first day.
error-gathering-ended = This gathering has ended and can no longer change.

members-title = Members
members-add = Add member
members-search = Search by name or mail address
members-filter = Show
members-filter-all = All
members-filter-active = Active
members-filter-inactive = Inactive
members-empty = No member matches.
members-count = { $count ->
        [one] One member
       *[other] { $count } members
    }

member-first-name = First name
member-last-name = Last name
member-name = Name
member-mail = Mail address
member-language = Language of mail
member-status = Status
member-active = Active
member-inactive = Inactive
member-organizer = Organizer
member-role = Role
member-organizer-hint = Organizers sign in and manage members and gatherings.
member-new-title = Add member
member-edit-title = Edit { $name }
member-save = Save
member-saved = Saved.
member-delete = Delete member
member-delete-confirm = Delete { $name } with all invitations, responses and waiting mail?
member-delete-yes = Delete
member-cancel = Cancel

error-field-required = Fill in this field.
error-field-too-long = This is too long.
error-mail-invalid = Enter a mail address such as name@example.org.
error-mail-taken = The address { $conflict } is already taken.
error-last-organizer = The instance needs at least one active organizer.
error-organizer-inactive = Only active members can be organizers.
error-unexpected = Something went wrong. Try again.

language-name-de = Deutsch
language-name-en = English

language-label = Language

spike-title = Response
spike-name = Name
spike-attendance = Attendance
spike-attendance-accepted = Accepted
spike-attendance-tentative = Tentative
spike-attendance-declined = Declined
spike-arrival = Arrival
spike-beds = Beds
spike-save = Save
spike-saved = Saved response of { $name }.

error-required = Enter a name.
error-too-long = The name is too long.
error-too-large = The number is too large.
error-taken = This name is already taken.
error-number = Enter a whole number.
error-date = Enter a valid date.
error-invalid = This value is not valid.
