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

invitation-mail-subject = Invitation: { $title }, { $span }
invitation-mail-greeting = Hello { $firstName },
invitation-mail-details = you are invited to { $title } in { $location }, { $span }.
invitation-mail-answer = Tell us through your personal link whether and when you come. Please do not forward it, because it works without a password.

cancellation-mail-subject = Cancelled: { $title }, { $span }
cancellation-mail-body = { $title } in { $location }, { $span }, does not take place.

sign-in-mail-subject = Your sign-in link for Adunata
sign-in-mail-body = Hello { $firstName }, open this link within one hour to sign in to Adunata. It works once.
sign-in-mail-ignore = If you did not ask to sign in, ignore this mail.

nav-label = Main navigation
nav-members = Members
nav-sign-out = Sign out
account-menu-label = Account of { $name }
account-own-entry = My member entry

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
gathering-first-day-months = Months for the first day
gathering-last-day-months = Months for the last day
date-picker-choose-day = Choose a day
gathering-save = Save
gathering-cancelled = Cancelled
error-last-day-before-first-day = The last day cannot lie before the first day.
error-first-day-in-past = The first day cannot lie in the past.
error-gathering-ended = This gathering has ended and can no longer change.

invitations-title = Invitations
invitations-invite-all = { $count ->
        [one] Invite the one active member not invited yet
       *[other] Invite all { $count } active members not invited yet
    }
invitations-choose = Choose members to invite
invitations-invite-chosen = { $count ->
        [0] Invite chosen members
        [one] Invite one chosen member
       *[other] Invite { $count } chosen members
    }
invitations-everyone-invited = Every active member is invited.
invitations-sent = { $count ->
        [0] Nobody new to invite.
        [one] One invitation sent.
       *[other] { $count } invitations sent.
    }
invitation-link-active = link works
invitation-link-revoked = link revoked
invitation-mail-waiting = mail waits
invitation-mail-sent = mail sent
invitation-mail-failed = mail failed
invitation-resend = Resend
invitation-revoke = Revoke link
invitation-resent = New link sent to { $name }.
invitation-revoked = The link of { $name } no longer works.
gathering-cancel = Cancel gathering
gathering-cancel-confirm = Cancel { $title } and inform every invited member by mail?
gathering-cancel-yes = Cancel gathering

response-days = Days of the gathering
invitation-invalid-title = Link not valid
invitation-invalid = This link is not valid. It may have been replaced by a newer one. Ask your organizers for a new link.
invitation-state-begun = The gathering has begun. Your response can no longer change here.
invitation-state-ended = The gathering is over.
invitation-state-cancelled = This gathering is cancelled.

overview-title = Responses
overview-total-accepted = Accepted
overview-total-tentative = Tentative
overview-total-declined = Declined
overview-total-pending = Pending
overview-beds-needed = Beds needed
overview-beds-offered = Beds offered
overview-bed-even = as many as needed
overview-bed-surplus = { $count ->
        [one] one more than needed
       *[other] { $count } more than needed
    }
overview-bed-shortage = { $count ->
        [one] one fewer than needed
       *[other] { $count } fewer than needed
    }
overview-filter = Filter by status
overview-filter-all = All { $count }
overview-filter-accepted = Accepted
overview-filter-tentative = Tentative
overview-filter-declined = Declined
overview-filter-pending = Pending
overview-column-name = Name
overview-column-status = Status
overview-column-period = Period
overview-column-beds = Beds
overview-column-comment = Comment
overview-column-actions = Actions
overview-beds-need = { $count ->
        [one] needs one
       *[other] needs { $count }
    }
overview-beds-offer = { $count ->
        [one] offers one
       *[other] offers { $count }
    }
overview-whole-gathering = whole gathering
overview-period = { $arrival } to { $departure }
overview-period-start = start
overview-period-end = end
overview-outside-span = outside the days

response-question = Are you coming, { $firstName }?
response-question-for = Is { $name } coming?
response-status-accepted = Yes
response-status-tentative = Maybe
response-status-declined = No
response-when = When are you there?
response-arrival = Arrival
response-departure = Departure
response-arrival-day = Day of arrival
response-departure-day = Day of departure
response-arrival-months = Months for arrival
response-departure-months = Months for departure
response-arrival-time = Time of arrival
response-departure-time = Time of departure
response-when-hint = { $days ->
        [one] Without a time we plan you for the whole day.
       *[other] Without times we plan you for all { $days } days.
    }
response-beds = Overnight stay
response-beds-need = I need a bed
response-beds-offer = I offer beds
response-beds-none = Neither
response-bed-count = How many?
response-comment = Comment
response-comment-hint = Only organizers read your comment.
response-save = Save response
response-saved = Response saved. You can change it until { $day }.
response-saved-for = Response of { $name } saved.
response-visibility = Other invited members see your name and whether you come.
response-yours = Your response
response-error-begun = The gathering has begun, so your response can no longer change here. Ask your organizers.
response-error-beds = Choose whether you need or offer beds.
response-error-bed-count = State how many beds, from 1 to 20.
response-error-bed-count-too-large = State at most 20 beds.
response-error-comment-too-long = The comment can have at most 2000 characters.
response-error-departure-before-arrival = Departure cannot lie before arrival.
response-error-time = Choose a time within the days of the gathering in steps of 30 minutes.
response-error-day-and-time = Choose both a day and a time.
attendees-title = { $tentative ->
        [0] Coming: { $accepted }
       *[other] Coming: { $accepted }, maybe { $tentative }
    }
attendees-more = and { $count } more
attendees-others = Declined: { $declined }. No response yet: { $pending }.
invitation-page-title = Invitation of { $name }
invitation-changed-by-member = changed by the member on { $at }
invitation-changed-by-organizer = changed by an organizer on { $at }
invitation-link-title = Personal link

members-title = Members
members-add = Add member
members-search = Search by name or mail address
members-filter = Show
members-filter-all = All
members-filter-active = Active
members-filter-inactive = Inactive
members-empty = No member matches.
members-copy-mail = Copy mail address
members-mail-copied = Mail address copied
members-copy-failed = Could not copy the mail address. Select and copy it by hand.
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
error-page-title = Page unavailable
error-retry = Try again

language-name-de = Deutsch
language-name-en = English

language-label = Language


error-required = Enter a name.
error-too-long = The name is too long.
error-too-large = The number is too large.
error-taken = This name is already taken.
error-number = Enter a whole number.
error-date = Enter a valid date.
error-invalid = This value is not valid.
