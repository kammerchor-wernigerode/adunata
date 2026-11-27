import { describe, expect, it } from 'vitest'
import { AnswerBody } from './generated/api.zod'
import type { ResponseValues } from './generated/model'

const valid: ResponseValues = {
  status: 'accepted',
  arrival: '2027-03-14T18:00',
  departure: null,
  beds: 'offer',
  bedCount: 2,
  car: true,
  freeSeats: 3,
  comment: null,
}

describe('generated contract', () => {
  it('accepts a complete response', () => {
    expect(AnswerBody.safeParse(valid).success).toBe(true)
  })

  it('accepts missing optional values', () => {
    expect(AnswerBody.safeParse({ status: 'declined' }).success).toBe(true)
  })

  it('rejects a value outside the enum', () => {
    expect(AnswerBody.safeParse({ ...valid, status: 'maybe' }).success).toBe(false)
  })

  it('rejects a fractional or out-of-range bed count', () => {
    expect(AnswerBody.safeParse({ ...valid, bedCount: 1.5 }).success).toBe(false)
    expect(AnswerBody.safeParse({ ...valid, bedCount: 21 }).success).toBe(false)
  })

  it('rejects free seats outside 0 to 8', () => {
    expect(AnswerBody.safeParse({ ...valid, freeSeats: 0 }).success).toBe(true)
    expect(AnswerBody.safeParse({ ...valid, freeSeats: 9 }).success).toBe(false)
    expect(AnswerBody.safeParse({ ...valid, freeSeats: -1 }).success).toBe(false)
  })
})
