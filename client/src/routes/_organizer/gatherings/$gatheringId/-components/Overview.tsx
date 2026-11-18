import { Check, Circle, CircleQuestionMark, X, type LucideIcon } from 'lucide-react'
import { useState } from 'react'
import { useOverview } from '@/api/generated/api'
import type { Gathering, InvitationStatus } from '@/api/generated/model'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { useTranslate, type Translate } from '@/l10n/translate'
import { dataOf } from '@/lib/dataOf'
import { OverviewTable } from './OverviewTable'

type Filter = 'all' | InvitationStatus
const statuses: InvitationStatus[] = ['accepted', 'tentative', 'declined', 'pending']
const icons: Record<InvitationStatus, LucideIcon> = {
  accepted: Check,
  tentative: CircleQuestionMark,
  declined: X,
  pending: Circle,
}

function bedBalance(translate: Translate, balance: number): string {
  if (balance > 0) return translate('overview-bed-surplus', { count: balance })
  if (balance < 0) return translate('overview-bed-shortage', { count: -balance })
  return translate('overview-bed-even')
}

/** Totals, filter and the table of every invitation of a gathering. */
export function Overview({ gathering }: { gathering: Gathering }) {
  const translate = useTranslate()
  const overview = useOverview(gathering.id)
  const [filter, setFilter] = useState<Filter>('all')
  const data = dataOf(overview.data)
  if (!data) return null
  const { rows, totals } = data
  const shown = filter === 'all' ? rows : rows.filter((row) => row.status === filter)

  return (
    <section aria-labelledby="overview-heading" className="mt-6">
      <h2 id="overview-heading" className="sr-only">
        {translate('overview-title')}
      </h2>
      <dl className="mb-4 grid grid-cols-[repeat(auto-fit,minmax(6.5rem,1fr))] overflow-hidden rounded-lg border border-border bg-card">
        {statuses.map((status) => {
          const Icon = icons[status]
          return (
            <div key={status} className="px-3 py-2 shadow-[-1px_0_0_var(--border)]">
              <dt className="text-sm text-muted-foreground">{translate(`overview-total-${status}`)}</dt>
              <dd className="flex items-center gap-1 font-heading text-2xl font-semibold tabular-nums">
                <Icon aria-hidden="true" strokeWidth={3} className="size-4 text-muted-foreground" />
                {totals[status]}
              </dd>
            </div>
          )
        })}
        <div className="px-3 py-2 shadow-[-1px_0_0_var(--border)]">
          <dt className="text-sm text-muted-foreground">{translate('overview-beds-needed')}</dt>
          <dd className="font-heading text-2xl font-semibold tabular-nums">{totals.bedsNeeded}</dd>
        </div>
        <div className="px-3 py-2 shadow-[-1px_0_0_var(--border)]">
          <dt className="text-sm text-muted-foreground">{translate('overview-beds-offered')}</dt>
          <dd className="font-heading text-2xl font-semibold tabular-nums">{totals.bedsOffered}</dd>
          <dd className="text-xs text-muted-foreground">
            {bedBalance(translate, totals.bedsOffered - totals.bedsNeeded)}
          </dd>
        </div>
      </dl>
      <ToggleGroup
        type="single"
        value={filter}
        onValueChange={(value) => value && setFilter(value as Filter)}
        aria-label={translate('overview-filter')}
        className="mb-3"
      >
        <ToggleGroupItem value="all">{translate('overview-filter-all', { count: rows.length })}</ToggleGroupItem>
        {statuses.map((status) => (
          <ToggleGroupItem key={status} value={status}>
            {translate(`overview-filter-${status}`)}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
      <OverviewTable gathering={gathering} rows={shown} />
    </section>
  )
}
