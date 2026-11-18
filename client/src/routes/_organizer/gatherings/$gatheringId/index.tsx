import { createFileRoute, Link } from '@tanstack/react-router'
import { useGetGathering } from '@/api/generated/api'
import { PageTitle } from '@/components/PageTitle'
import { Button } from '@/components/ui/button'
import { formatSpan } from '@/l10n/dates'
import { useLocale, useTranslate } from '@/l10n/translate'
import { dataOf } from '@/lib/dataOf'
import { InvitationActions } from './-components/InvitationActions'
import { Overview } from './-components/Overview'

export const Route = createFileRoute('/_organizer/gatherings/$gatheringId/')({ component: GatheringPage })

/** The organizer's view of one gathering. It opens with title, span and location and the action to edit it. */
function GatheringPage() {
  const translate = useTranslate()
  const { locale } = useLocale()
  const { gatheringId } = Route.useParams()
  const gathering = useGetGathering(Number(gatheringId))
  const data = dataOf(gathering.data)
  if (!data) return null
  const { title, firstDay, lastDay, location, description, cancelled, ended } = data

  return (
    <>
      <PageTitle title={title} />
      <div className="-mt-2 mb-6 flex flex-wrap items-end justify-between gap-4">
        <div className="min-w-0">
          <p className="text-muted-foreground">
            {formatSpan(firstDay, lastDay, locale)}, {location}
          </p>
          {description && <p className="mt-2 max-w-prose whitespace-pre-line">{description}</p>}
          {cancelled && <p className="mt-2 font-semibold">{translate('gathering-cancelled')}</p>}
        </div>
        {!ended && !cancelled && (
          <Button asChild variant="outline">
            <Link to="/gatherings/$gatheringId/edit" params={{ gatheringId }}>
              {translate('gathering-edit')}
            </Link>
          </Button>
        )}
      </div>
      <Overview gathering={data} />
      <InvitationActions gathering={data} />
    </>
  )
}
