import { useState } from 'react'
import type { Attendees as AttendeeList } from '@/api/generated/model'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { useTranslate } from '@/l10n/translate'
import { cn } from '@/lib/utils'

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
          <li key={attendee.name}>
            <Badge
              variant={attendee.status === 'tentative' ? 'outline' : 'secondary'}
              className={cn(
                'h-auto px-3 text-sm font-normal',
                attendee.status === 'tentative' ? 'border-dashed border-muted-foreground' : 'bg-muted',
              )}
            >
              {attendee.name}
              {attendee.status === 'tentative' && <span className="sr-only"> ({translate('response-status-tentative')})</span>}
            </Badge>
          </li>
        ))}
        {hidden > 0 && (
          <li>
            <Button variant="secondary" size="sm" onClick={() => setAll(true)} className="h-auto rounded-full bg-muted px-3 py-0.5 underline underline-offset-4">
              {translate('attendees-more', { count: hidden })}
            </Button>
          </li>
        )}
      </ul>
      <p className="mt-2 text-sm text-muted-foreground">
        {translate('attendees-others', { declined: attendees.declined, pending: attendees.pending })}
      </p>
    </section>
  )
}
