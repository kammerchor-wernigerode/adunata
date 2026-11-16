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
|Trusted proxy|A reverse proxy whose network the operator names in `ADUNATA_TRUSTED_PROXIES`. Adunata takes the network address of a client from its `X-Forwarded-For` header.|
|Personal link|The secret link in an invitation mail. It lets the invited member answer without an account.|
|Response|A member's answer to an invitation, with status, period of presence, beds and comment.|
|Project|A term many ensembles use for a series of gatherings that leads to a concert, such as a Christmas project. Adunata does not model projects. A project is never a gathering.|
|Instance|One installation of Adunata, run by one ensemble.|
|Operator|The person who installs, configures and updates an instance.|
|Outbox|The table in which Adunata stores each mail before a background task sends it.|
