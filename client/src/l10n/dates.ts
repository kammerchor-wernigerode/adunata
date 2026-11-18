import { parseISO } from 'date-fns'

/** Formats an ISO date such as 2027-03-14 in the reader's language. The date stays the same day in every time zone, because it is parsed as local time. */
export function formatDay(isoDate: string, locale: string, options: Intl.DateTimeFormatOptions = { dateStyle: 'medium' }) {
  return new Intl.DateTimeFormat(locale, options).format(parseISO(isoDate))
}

/** Formats the span from the first to the last day of a gathering. */
export function formatSpan(firstDay: string, lastDay: string, locale: string) {
  return new Intl.DateTimeFormat(locale, { weekday: 'short', day: 'numeric', month: 'long', year: 'numeric' }).formatRange(
    parseISO(firstDay),
    parseISO(lastDay),
  )
}

/** Builds a local date from a wall-clock time such as 2027-03-14T18:00 without converting it between time zones. */
export function wallClock(value: string): Date {
  return parseISO(value)
}

/** Formats a wall-clock time with weekday and time, such as "Fri 18:00". */
export function formatWallClock(value: string, locale: string) {
  return new Intl.DateTimeFormat(locale, { weekday: 'short', hour: '2-digit', minute: '2-digit', hourCycle: 'h23' }).format(wallClock(value))
}
