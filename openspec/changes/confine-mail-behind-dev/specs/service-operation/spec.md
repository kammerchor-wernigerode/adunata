## ADDED Requirements

### Requirement: Warning while mail is confined
`serve` SHALL log a warning at start while mail confinement is on, before it accepts requests. The warning SHALL say that every mail goes to the redirect address. It SHALL not hold the redirect address.

#### Scenario: Start with confinement on
- **WHEN** an operator starts `serve --dev` with `ADNT_MAIL_CONFINE=true` and `ADNT_MAIL_CONFINE_TO=test@example.org`
- **THEN** the log holds one entry at level warning that says mail confinement is on, and no log entry holds `test@example.org`

#### Scenario: Start with confinement off
- **WHEN** an operator starts `serve --dev` without the confinement settings
- **THEN** the log holds no warning about mail confinement
