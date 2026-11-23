import { createFileRoute, Link } from '@tanstack/react-router'
import { useGetGathering } from '@/api/generated/api'
import { PageTitle } from '@/components/PageTitle'
import { Button } from '@/components/ui/button'
import { formatDay, formatSpan } from '@/l10n/dates'
import { useLocale, useTranslate } from '@/l10n/translate'
import { dataOf } from '@/lib/dataOf'
import { InvitationActions } from './-components/InvitationActions'
import { Overview } from './-components/Overview'

export const Route = createFileRoute('/_organizer/gatherings/$gatheringId/')({ component: GatheringPage })

/** The organizer's view of one gathering. It opens with title and the action to edit it, followed by span and location. */
function GatheringPage() {
  const translate = useTranslate()
  const { locale } = useLocale()
  const { gatheringId } = Route.useParams()
  const gathering = useGetGathering(Number(gatheringId))
  const data = dataOf(gathering.data)
  if (!data) return null
  const { title, firstDay, lastDay, location, description, cancelled, ended, lastDayToAnswer, deadlinePassed } = data

  return (
    <>
      <PageTitle
        title={title}
        action={
          !ended &&
          !cancelled && (
            <Button asChild variant="outline">
              <Link to="/gatherings/$gatheringId/edit" params={{ gatheringId }}>
                {translate('gathering-edit')}
              </Link>
            </Button>
          )
        }
      />
      <div className="-mt-2 mb-6 min-w-0">
        <p className="text-muted-foreground">
          {formatSpan(firstDay, lastDay, locale)}, {location}
        </p>
        {lastDayToAnswer && (
          <p className="text-muted-foreground">
            {translate(deadlinePassed ? 'gathering-deadline-passed' : 'gathering-deadline-ahead', {
              day: formatDay(lastDayToAnswer, locale),
            })}
          </p>
        )}
        {description && <p className="mt-2 max-w-prose whitespace-pre-line">{description}</p>}
        {cancelled && <p className="mt-2 font-semibold">{translate('gathering-cancelled')}</p>}
      </div>
      <Overview gathering={data} />
      <InvitationActions gathering={data} />
    </>
  )
}
