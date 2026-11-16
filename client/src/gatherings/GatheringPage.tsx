import { Link, Navigate, useParams } from '@tanstack/react-router'
import { useGetGathering } from '@/api/generated/api'
import { Button } from '@/components/ui/button'
import { Page } from '@/layout/Page'
import { InvitationActions } from './InvitationActions'
import { formatSpan } from '@/l10n/dates'
import { useLocale, useTranslate } from '@/l10n/translate'

/** The organizer's view of one gathering. It opens with title, span and location and the action to edit it. */
export function GatheringPage() {
  const translate = useTranslate()
  const { locale } = useLocale()
  const { gatheringId } = useParams({ from: '/organizer/gatherings/$gatheringId' })
  const gathering = useGetGathering(Number(gatheringId))
  if (gathering.data?.status === 401) return <Navigate to="/sign-in" />
  if (gathering.data?.status === 404) return <Navigate to="/gatherings" />
  if (gathering.data?.status !== 200) return null
  const { title, firstDay, lastDay, location, description, cancelled, ended } = gathering.data.data

  return (
    <Page title={title}>
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
      <InvitationActions gathering={gathering.data.data} />
    </Page>
  )
}
