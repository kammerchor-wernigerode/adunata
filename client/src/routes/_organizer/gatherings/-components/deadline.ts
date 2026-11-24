import { format, isValid, parseISO, subDays } from 'date-fns'

export type DeadlineUnit = 'days' | 'weeks'
const daysPerUnit: Record<DeadlineUnit, number> = { days: 1, weeks: 7 }

/** The deadline in days that a number and a unit stand for, null when the number is empty and undefined when it is no whole number. */
export function deadlineDays(amount: string, unit: DeadlineUnit): number | null | undefined {
  const trimmed = amount.trim()
  if (trimmed === '') return null
  if (!/^\d+$/.test(trimmed)) return undefined
  return Number(trimmed) * daysPerUnit[unit]
}

/** The last day to answer, the first day minus the deadline minus one day, or an empty string when no ISO date can name that day. */
export function lastDayToAnswer(firstDay: string, days: number): string {
  const day = subDays(parseISO(firstDay), days + 1)
  if (!isValid(day)) return ''
  const iso = format(day, 'yyyy-MM-dd')
  return isValid(parseISO(iso)) ? iso : ''
}

/** Why a deadline cannot be saved, or undefined when it can. A last day to answer before today is refused unless it equals the stored one. */
export function deadlineProblem(
  days: number | null | undefined,
  firstDay: string,
  today: string,
  stored: string | null | undefined,
): 'invalid' | 'deadlineInPast' | undefined {
  if (days === null || !firstDay) return undefined
  if (days === undefined || days < 1) return 'invalid'
  const lastDay = lastDayToAnswer(firstDay, days)
  if ((lastDay === '' || lastDay < today) && lastDay !== stored) return 'deadlineInPast'
  return undefined
}
