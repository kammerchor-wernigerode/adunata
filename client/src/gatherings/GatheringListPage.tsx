import { Link, Navigate } from '@tanstack/react-router'
import { useListGatherings } from '@/api/generated/api'
import type { Gathering } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import { Page } from '@/layout/Page'
import { formatSpan } from '@/l10n/dates'
import { useLocale, useTranslate } from '@/l10n/translate'

export function GatheringListPage() {
  const translate = useTranslate()
  const gatherings = useListGatherings()
  if (gatherings.data?.status === 401) return <Navigate to="/sign-in" />
  const list = gatherings.data?.status === 200 ? gatherings.data.data : null

  return (
    <Page title={translate('gatherings-title')}>
      <Button asChild className="mb-6">
        <Link to="/gatherings/new">{translate('gatherings-create')}</Link>
      </Button>
      {list && (
        <>
          <GatheringSection title={translate('gatherings-upcoming')} gatherings={list.upcoming} />
          <GatheringSection title={translate('gatherings-past')} gatherings={list.past} />
        </>
      )}
    </Page>
  )
}

function GatheringSection({ title, gatherings }: { title: string; gatherings: Gathering[] }) {
  const translate = useTranslate()
  const { locale } = useLocale()
  return (
    <section className="mb-8">
      <h2 className="mb-3 font-heading text-lg font-semibold">{title}</h2>
      {gatherings.length === 0 ? (
        <p className="text-muted-foreground">{translate('gatherings-none')}</p>
      ) : (
        <ul className="divide-y divide-border overflow-hidden rounded-lg border border-border">
          {gatherings.map((gathering) => (
            <li key={gathering.id} className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 px-4 py-3">
              <div className="min-w-0">
                <Link
                  to="/gatherings/$gatheringId"
                  params={{ gatheringId: String(gathering.id) }}
                  className="font-semibold text-foreground underline underline-offset-4"
                >
                  {gathering.title}
                </Link>
                <p className="text-sm text-muted-foreground">
                  {formatSpan(gathering.firstDay, gathering.lastDay, locale)}, {gathering.location}
                </p>
              </div>
              {gathering.cancelled && (
                <span className="rounded-full bg-muted px-2.5 py-0.5 text-sm font-semibold text-muted-foreground">
                  {translate('gathering-cancelled')}
                </span>
              )}
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}
