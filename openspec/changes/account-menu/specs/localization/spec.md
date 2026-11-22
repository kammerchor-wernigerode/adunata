## MODIFIED Requirements

### Requirement: Language of the browser client
Adunata SHALL show the browser client in the language that the browser prefers among the supported languages. A person SHALL be able to switch the language. With a session the switch SHALL be in the account menu. Without a session it SHALL be in the header. The client SHALL remember that choice on the device. Switching SHALL not change the language stored for the member's mail.

#### Scenario: German browser
- **WHEN** a browser that prefers German opens the client for the first time
- **THEN** the client shows German text

#### Scenario: Unsupported browser language
- **WHEN** a browser that prefers only French opens the client
- **THEN** the client shows English text

#### Scenario: Switch language
- **WHEN** a person switches the client to English and reloads the page
- **THEN** the client still shows English text

#### Scenario: Switch language without a session
- **WHEN** a person without a session views the sign-in page
- **THEN** the header shows a language select with Deutsch and English and choosing one switches the client at once

#### Scenario: Switch language in the account menu
- **WHEN** a signed-in organizer chooses Deutsch in the account menu and reloads the page
- **THEN** the client shows German text, the menu marks Deutsch as chosen, the header shows no separate language select and the member's stored language of mail is unchanged
