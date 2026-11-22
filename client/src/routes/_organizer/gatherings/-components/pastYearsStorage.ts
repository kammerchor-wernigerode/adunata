/** Whether the organizer left each year open (`true`) or closed (`false`), by year. */
export type YearChoices = Record<string, boolean>

const storageKey = 'adunata.past-years'

function isYearChoices(value: unknown): value is YearChoices {
  return (
    typeof value === 'object' &&
    value !== null &&
    !Array.isArray(value) &&
    Object.values(value).every((open) => typeof open === 'boolean')
  )
}

export function readYearChoices(): YearChoices {
  try {
    const stored: unknown = JSON.parse(localStorage.getItem(storageKey) ?? '{}')
    return isYearChoices(stored) ? stored : {}
  } catch {
    return {}
  }
}

export function storeYearChoices(choices: YearChoices): void {
  try {
    localStorage.setItem(storageKey, JSON.stringify(choices))
  } catch {
    // The choices then last only until the page reloads.
  }
}
