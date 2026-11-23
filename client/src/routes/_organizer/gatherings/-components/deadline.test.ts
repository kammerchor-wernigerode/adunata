import { describe, expect, it } from 'vitest'
import { deadlineDays, lastDayToAnswer } from './deadline'

describe('deadline', () => {
  it('counts 7 days per week', () => {
    expect(deadlineDays('2', 'weeks')).toBe(14)
    expect(deadlineDays('7', 'days')).toBe(7)
  })

  it('means no deadline for an empty number', () => {
    expect(deadlineDays(' ', 'days')).toBeNull()
  })

  it('refuses a number that is not whole', () => {
    expect(deadlineDays('1.5', 'days')).toBeUndefined()
    expect(deadlineDays('-1', 'days')).toBeUndefined()
  })

  it('lies the deadline and one more day before the first day', () => {
    expect(lastDayToAnswer('2027-03-14', 7)).toBe('2027-03-06')
    expect(lastDayToAnswer('2027-03-14', 1)).toBe('2027-03-12')
  })
})
