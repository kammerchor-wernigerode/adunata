import type { Translate } from '@/l10n/translate'

/** The car of a response as text: its free seats, "no car" or a dash without a statement. */
export function carSummary(translate: Translate, { car, freeSeats }: { car?: boolean | null; freeSeats?: number | null }): string {
  if (car === true) return translate('overview-free-seats', { count: freeSeats ?? 0 })
  if (car === false) return translate('overview-car-none')
  return '–'
}
