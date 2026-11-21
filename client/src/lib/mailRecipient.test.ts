import { describe, expect, it } from 'vitest'
import { mailRecipient } from './mailRecipient'

const mail = 'ada@example.org'

describe('mailRecipient', () => {
  it('leaves a plain name unquoted', () => {
    expect(mailRecipient({ firstName: 'Ada', lastName: 'Lovelace', mail })).toBe('Ada Lovelace <ada@example.org>')
  })

  it('quotes a name with a comma', () => {
    expect(mailRecipient({ firstName: 'Ada', lastName: 'Lovelace, Countess', mail })).toBe('"Ada Lovelace, Countess" <ada@example.org>')
  })

  it('quotes a name with a double quote and escapes it', () => {
    expect(mailRecipient({ firstName: 'Ada "Bird"', lastName: 'Lovelace', mail })).toBe('"Ada \\"Bird\\" Lovelace" <ada@example.org>')
  })

  it('escapes a backslash in a quoted name', () => {
    expect(mailRecipient({ firstName: 'Ada\\', lastName: 'Lovelace, Countess', mail })).toBe('"Ada\\\\ Lovelace, Countess" <ada@example.org>')
  })

  it('leaves a backslash alone in an unquoted name', () => {
    expect(mailRecipient({ firstName: 'Ada\\', lastName: 'Lovelace', mail })).toBe('Ada\\ Lovelace <ada@example.org>')
  })
})
