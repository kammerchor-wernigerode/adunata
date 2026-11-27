# Glossary

|Term|Definition|
|-|-|
|Ensemble|A choir or an instrumental ensemble that uses Adunata.|
|Member|A person who belongs to an ensemble and can be invited to gatherings. A member has a first and a last name, a mail address that no other member has in any case, a language for mail and is active or inactive. Only active members receive new invitations.|
|Organizer|An active member with the organizer role. Organizers keep the member list, create gatherings and invite members.|
|Organizer role|The right to use organizer functions. Organizers grant it to active members and revoke it. An instance always keeps at least one active organizer.|
|Gathering|A continuous span of one or more days on which an ensemble meets to rehearse or to give concerts, such as a rehearsal weekend, a rehearsal camp during the week or a concert.|
|Invitation|The request to one member to attend one gathering. Each invitation has its own personal link.|
|Organizer session|The signed-in state of an organizer in one browser. It ends after 14 days, on sign-out or when the member loses the role or is deactivated.|
|Sign-in link|The single-use link that Adunata mails to an organizer who wants to sign in. It works once within one hour and only after the organizer presses a button.|
|Trusted proxy|A reverse proxy whose network the operator names in `ADNT_HTTP_TRUSTED_PROXIES`. Adunata takes the network address of a client from its `X-Forwarded-For` header.|
|Personal link|The secret link in an invitation mail. It lets the invited member answer without an account.|
|Period of presence|When a member who comes arrives and leaves, as wall-clock times of the instance in steps of 30 minutes within the days of a gathering. Without a stated period the member is present for the whole gathering.|
|Tentative|The status of a member who will probably come but cannot promise it yet. Tentative attendees count separately from those who accepted.|
|Retention period|The number of days after the last day of a gathering for which Adunata keeps comments or responses. Operators set each period or keep the data.|
|Response|A member's answer to an invitation, with status, period of presence, beds, car and comment.|
|Project|A term many ensembles use for a series of gatherings that leads to a concert, such as a Christmas project. Adunata does not model projects. A project is never a gathering.|
|Instance|One installation of Adunata, run by one ensemble.|
|Operator|The person who installs, configures and updates an instance.|
|Outbox|The table in which Adunata stores each mail before a background task sends it.|
|First organizer|The organizer whom the operator names in the configuration, so that someone can sign in to a new instance.|
|Reminder|The mail that an organizer sends to every member of a gathering who has not answered. It carries a new personal link.|
|Cancellation|The act of an organizer that marks a gathering as cancelled and informs every invited member by mail. A cancelled gathering stays visible to organizers.|
|Revocation|The act of an organizer that makes a personal link invalid without sending a new one.|
|Deadline for responses|The optional number of days before the first day of a gathering after which a member can only decline.|
|Beds|Whether a member who comes needs beds, offers beds or needs none, with their number.|
|Car|Whether a member who comes arrives by car, and with a car the free seats from 0 to 8, not counting the driver. Only organizers see it.|
|Calendar file|The file in iCalendar format that describes a gathering. Members download it from the response page and receive it with mail about the gathering.|
|Local mode|The mode of `adunata serve --local` for debug runs on `localhost`, which writes sign-in links to the log.|
|Mail confinement|The state in which Adunata sends every mail to one redirect address instead of its recipient. Two settings turn it on, and only `adunata serve --dev` accepts them.|
|Feature module|A module of the server that owns one part of the domain with its routes, its rules and its tables.|
|Technical module|A module of the server that solves one problem for several feature modules.|
