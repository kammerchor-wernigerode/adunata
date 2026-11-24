import { describe, expect, it } from 'vitest'
import { deadlineDays, deadlineProblem, lastDayToAnswer } from './deadline'

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

  it('gives no day outside the range of Date', () => {
    expect(lastDayToAnswer('2027-03-14', 200_000_000)).toBe('')
    expect(lastDayToAnswer('2027-03-14', 30_000_000)).toBe('')
  })

  it('refuses zero, a past last day and a huge number but keeps the stored day', () => {
    expect(deadlineProblem(0, '2027-03-14', '2027-03-01', null)).toBe('invalid')
    expect(deadlineProblem(7, '2027-03-14', '2027-03-06', null)).toBeUndefined()
    expect(deadlineProblem(7, '2027-03-14', '2027-03-07', null)).toBe('deadlineInPast')
    expect(deadlineProblem(7, '2027-03-14', '2027-03-07', '2027-03-06')).toBeUndefined()
    expect(deadlineProblem(200_000_000, '2027-03-14', '2027-03-01', null)).toBe('deadlineInPast')
    expect(deadlineProblem(null, '2027-03-14', '2027-03-07', null)).toBeUndefined()
  })

  it('places the last day to answer the deadline and one more day before the first day', () => {
    expect(lastDayToAnswer('2027-03-14', 7)).toBe('2027-03-06')
    expect(lastDayToAnswer('2027-03-14', 1)).toBe('2027-03-12')
  })
})
