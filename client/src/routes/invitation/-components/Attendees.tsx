import { useState } from 'react'
import type { Attendees as AttendeeList } from '@/api/generated/model'
import { useTranslate } from '@/l10n/translate'

const shownAtFirst = 12

/** Names of everyone who comes as chips. A tentative attendee has a dashed outline and a hidden word for screen readers. */
export function Attendees({ attendees }: { attendees: AttendeeList }) {
  const translate = useTranslate()
  const [all, setAll] = useState(false)
  const accepted = attendees.coming.filter((attendee) => attendee.status === 'accepted').length
  const tentative = attendees.coming.length - accepted
  const shown = all || attendees.coming.length <= shownAtFirst ? attendees.coming : attendees.coming.slice(0, shownAtFirst - 1)
  const hidden = attendees.coming.length - shown.length

  return (
    <section aria-labelledby="attendees-heading" className="mt-8 border-t border-border pt-4">
      <h2 id="attendees-heading" className="mb-2 font-heading text-lg font-semibold">
        {translate('attendees-title', { accepted, tentative })}
      </h2>
      <ul className="flex flex-wrap gap-2">
        {shown.map((attendee) => (
          <li
            key={attendee.name}
            className={
              attendee.status === 'tentative'
                ? 'rounded-full border border-dashed border-muted-foreground px-3 py-0.5 text-sm'
                : 'rounded-full bg-muted px-3 py-0.5 text-sm'
            }
          >
            {attendee.name}
            {attendee.status === 'tentative' && <span className="sr-only"> ({translate('response-status-tentative')})</span>}
          </li>
        ))}
        {hidden > 0 && (
          <li>
            <button
              type="button"
              onClick={() => setAll(true)}
              className="rounded-full bg-muted px-3 py-0.5 text-sm underline underline-offset-4"
            >
              {translate('attendees-more', { count: hidden })}
            </button>
          </li>
        )}
      </ul>
      <p className="mt-2 text-sm text-muted-foreground">
        {translate('attendees-others', { declined: attendees.declined, pending: attendees.pending })}
      </p>
    </section>
  )
}
