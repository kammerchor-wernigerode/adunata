# Spec Delta

## Purpose

Shows every text in the browser client and in every mail in the language of the person who reads it, starting with German and English.

## ADDED Requirements

### Requirement: Supported languages
Adunata SHALL offer German and English. English SHALL be the fallback language for any text that a catalog lacks.

#### Scenario: Missing translation
- **WHEN** the German catalog lacks a text that the English catalog has
- **THEN** Adunata shows the English text instead of a key or an empty string

### Requirement: Text from catalogs
Adunata SHALL take every user-visible text in the browser client and in mail from a translation catalog. Texts SHALL support placeholders and plural rules of each language.

#### Scenario: Plural form
- **WHEN** a text states a count, such as the number of attendees
- **THEN** Adunata uses the grammatically correct form for that count in the shown language

### Requirement: Language of the browser client
Adunata SHALL show the browser client in the language that the browser prefers among the supported languages. A person SHALL be able to switch the language. The client SHALL remember that choice on the device.

#### Scenario: German browser
- **WHEN** a browser that prefers German opens the client for the first time
- **THEN** the client shows German text

#### Scenario: Unsupported browser language
- **WHEN** a browser that prefers only French opens the client
- **THEN** the client shows English text

#### Scenario: Switch language
- **WHEN** a person switches the client to English and reloads the page
- **THEN** the client still shows English text

### Requirement: Language of mail
Adunata SHALL write each mail in the language stored for its recipient. Without a stored language it SHALL use the default language of the instance.

#### Scenario: Recipient with a stored language
- **WHEN** Adunata sends a mail to a recipient whose stored language is English
- **THEN** the subject and the body are in English

#### Scenario: Recipient without a stored language
- **WHEN** Adunata sends a mail to a recipient without a stored language
- **THEN** the mail uses the configured default language of the instance

### Requirement: Localized dates and times
Adunata SHALL format dates, times and date ranges according to the shown language. It SHALL show times in the time zone of the instance.

#### Scenario: Date range in German
- **WHEN** the client shows a gathering from 14 to 16 March 2027 in German
- **THEN** it formats the range the way German readers expect, with day before month
