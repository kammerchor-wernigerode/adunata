import { describe, expect, it } from 'vitest'
import { CreateResponseBody } from './generated/api.zod'
import type { SpikeResponse } from './generated/model'

const valid: SpikeResponse = {
  name: 'Ada',
  attendance: 'accepted',
  arrival: '2027-03-14',
  beds: 2,
}

describe('generated contract', () => {
  it('accepts a complete response', () => {
    expect(CreateResponseBody.safeParse(valid).success).toBe(true)
  })

  it('accepts a missing or null optional date', () => {
    const { arrival: _, ...withoutArrival } = valid
    expect(CreateResponseBody.safeParse(withoutArrival).success).toBe(true)
    expect(CreateResponseBody.safeParse({ ...valid, arrival: null }).success).toBe(true)
  })

  it('rejects a malformed date', () => {
    expect(CreateResponseBody.safeParse({ ...valid, arrival: '14.03.2027' }).success).toBe(false)
  })

  it('rejects a value outside the enum', () => {
    expect(CreateResponseBody.safeParse({ ...valid, attendance: 'maybe' }).success).toBe(false)
  })

  it('rejects a fractional or out-of-range integer', () => {
    expect(CreateResponseBody.safeParse({ ...valid, beds: 1.5 }).success).toBe(false)
    expect(CreateResponseBody.safeParse({ ...valid, beds: 21 }).success).toBe(false)
  })
})
