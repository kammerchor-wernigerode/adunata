/** Formats an ISO date such as 2027-03-14 in the reader's language. The date stays the same day in every time zone, because it is built from its parts. */
export function formatDay(isoDate: string, locale: string, options: Intl.DateTimeFormatOptions = { dateStyle: 'medium' }) {
  const [year, month, day] = isoDate.split('-').map(Number)
  return new Intl.DateTimeFormat(locale, options).format(new Date(year, month - 1, day))
}

/** Formats the span from the first to the last day of a gathering. */
export function formatSpan(firstDay: string, lastDay: string, locale: string) {
  const [startYear, startMonth, startDay] = firstDay.split('-').map(Number)
  const [endYear, endMonth, endDay] = lastDay.split('-').map(Number)
  return new Intl.DateTimeFormat(locale, { weekday: 'short', day: 'numeric', month: 'long', year: 'numeric' }).formatRange(
    new Date(startYear, startMonth - 1, startDay),
    new Date(endYear, endMonth - 1, endDay),
  )
}

/** Builds a local date from a wall-clock time such as 2027-03-14T18:00 without converting it between time zones. */
export function wallClock(value: string): Date {
  const [day, time] = value.split('T')
  const [year, month, date] = day.split('-').map(Number)
  const [hour, minute] = time.split(':').map(Number)
  return new Date(year, month - 1, date, hour, minute)
}

/** Formats a wall-clock time with weekday and time, such as "Fri 18:00". */
export function formatWallClock(value: string, locale: string) {
  return new Intl.DateTimeFormat(locale, { weekday: 'short', hour: '2-digit', minute: '2-digit', hourCycle: 'h23' }).format(wallClock(value))
}
