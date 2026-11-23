import { format, parseISO, subDays } from 'date-fns'

export type DeadlineUnit = 'days' | 'weeks'
const daysPerUnit: Record<DeadlineUnit, number> = { days: 1, weeks: 7 }

/** The deadline in days that a number and a unit stand for, null when the number is empty and undefined when it is no whole number. */
export function deadlineDays(amount: string, unit: DeadlineUnit): number | null | undefined {
  const trimmed = amount.trim()
  if (trimmed === '') return null
  if (!/^\d+$/.test(trimmed)) return undefined
  return Number(trimmed) * daysPerUnit[unit]
}

/** The last day to answer, the first day minus the deadline minus one day. */
export function lastDayToAnswer(firstDay: string, days: number): string {
  return format(subDays(parseISO(firstDay), days + 1), 'yyyy-MM-dd')
}
