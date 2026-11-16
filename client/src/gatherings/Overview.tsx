import { useQueryClient } from '@tanstack/react-query'
import { Check, Circle, CircleQuestionMark, X, type LucideIcon } from 'lucide-react'
import { useState } from 'react'
import { getOverviewQueryKey, resend, useOverview } from '@/api/generated/api'
import type { Gathering, InvitationStatus, OverviewRow } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { formatWallClock, wallClock } from '@/l10n/dates'
import { useLocale, useTranslate } from '@/l10n/translate'
import { StatusBadge } from '@/responses/StatusBadge'

type Filter = 'all' | InvitationStatus
const statuses: InvitationStatus[] = ['accepted', 'tentative', 'declined', 'pending']
const icons: Record<InvitationStatus, LucideIcon> = {
  accepted: Check,
  tentative: CircleQuestionMark,
  declined: X,
  pending: Circle,
}

/** Totals, filter and the table of every invitation of a gathering. */
export function Overview({ gathering }: { gathering: Gathering }) {
  const translate = useTranslate()
  const overview = useOverview(gathering.id)
  const [filter, setFilter] = useState<Filter>('all')
  if (overview.data?.status !== 200) return null
  const { rows, totals } = overview.data.data
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

const cell = 'px-3 py-2 align-top @max-[40rem]:flex @max-[40rem]:gap-3 @max-[40rem]:px-0 @max-[40rem]:py-0.5'
const label = 'sr-only @max-[40rem]:not-sr-only @max-[40rem]:min-w-24 @max-[40rem]:text-sm @max-[40rem]:text-muted-foreground'

function OverviewTable({ gathering, rows }: { gathering: Gathering; rows: OverviewRow[] }) {
  const translate = useTranslate()
  const columns = ['name', 'status', 'period', 'beds', 'comment'] as const
  return (
    <div className="@container">
      <table className="w-full border-collapse overflow-hidden rounded-lg border border-border bg-card text-sm @max-[40rem]:block">
        <thead className="bg-muted text-left text-xs text-muted-foreground @max-[40rem]:hidden">
          <tr>
            {columns.map((column) => (
              <th key={column} scope="col" className="px-3 py-2 font-semibold">
                {translate(`overview-column-${column}`)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="@max-[40rem]:block">
          {rows.map((row) => (
            <tr key={row.invitationId} className="border-t border-border even:bg-muted/50 @max-[40rem]:block @max-[40rem]:px-3 @max-[40rem]:py-2">
              <td className={`${cell} font-semibold`}>
                <span className={label}>{translate('overview-column-name')}</span>
                <span>
                  {row.firstName} {row.lastName}
                  <MailNotice row={row} gathering={gathering} />
                </span>
              </td>
              <td className={cell}>
                <span className={label}>{translate('overview-column-status')}</span>
                <StatusBadge status={row.status} />
              </td>
              <td className={`${cell} whitespace-nowrap @max-[40rem]:whitespace-normal`}>
                <span className={label}>{translate('overview-column-period')}</span>
                <Period gathering={gathering} row={row} />
              </td>
              <td className={cell}>
                <span className={label}>{translate('overview-column-beds')}</span>
                <span>{beds(translate, row)}</span>
              </td>
              <td className={`${cell} text-muted-foreground`}>
                <span className={label}>{translate('overview-column-comment')}</span>
                <span className="break-words">{row.comment ?? '–'}</span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

function bedBalance(translate: ReturnType<typeof useTranslate>, balance: number): string {
  if (balance > 0) return translate('overview-bed-surplus', { count: balance })
  if (balance < 0) return translate('overview-bed-shortage', { count: -balance })
  return translate('overview-bed-even')
}

function beds(translate: ReturnType<typeof useTranslate>, row: OverviewRow): string {
  if (row.beds === 'need') return translate('overview-beds-need', { count: row.bedCount ?? 0 })
  if (row.beds === 'offer') return translate('overview-beds-offer', { count: row.bedCount ?? 0 })
  return '–'
}

function MailNotice({ row, gathering }: { row: OverviewRow; gathering: Gathering }) {
  const translate = useTranslate()
  const queryClient = useQueryClient()
  const [sent, setSent] = useState(false)
  if (row.mail !== 'failed' && row.linkActive) return null
  const open = !gathering.cancelled && !gathering.ended
  return (
    <span className="mt-1 flex flex-wrap items-center gap-2 text-sm font-normal text-muted-foreground">
      {row.mail === 'failed' ? translate('invitation-mail-failed') : translate('invitation-link-revoked')}
      {open && !sent && (
        <Button
          size="sm"
          variant="outline"
          onClick={async () => {
            await resend(row.invitationId)
            setSent(true)
            await queryClient.invalidateQueries({ queryKey: getOverviewQueryKey(gathering.id) })
          }}
        >
          {translate('invitation-resend')}
          <span className="sr-only">
            : {row.firstName} {row.lastName}
          </span>
        </Button>
      )}
    </span>
  )
}

/** Shows the stated period as text with a bar across the days of the gathering. Without a period the member is present throughout. */
function Period({ gathering, row }: { gathering: Gathering; row: OverviewRow }) {
  const translate = useTranslate()
  const { locale } = useLocale()
  if (row.status === 'declined' || row.status === 'pending') return <span>–</span>
  const start = wallClock(`${gathering.firstDay}T00:00`)
  const end = wallClock(`${gathering.lastDay}T00:00`)
  end.setDate(end.getDate() + 1)
  const span = end.getTime() - start.getTime()
  const days = Math.round(span / 86_400_000)
  const share = (value: string | null | undefined, fallback: number) =>
    value ? Math.min(100, Math.max(0, ((wallClock(value).getTime() - start.getTime()) / span) * 100)) : fallback
  const from = share(row.arrival, 0)
  const to = share(row.departure, 100)
  const text =
    row.arrival || row.departure
      ? translate('overview-period', {
          arrival: row.arrival ? formatWallClock(row.arrival, locale) : translate('overview-period-start'),
          departure: row.departure ? formatWallClock(row.departure, locale) : translate('overview-period-end'),
        })
      : translate('overview-whole-gathering')
  return (
    <span className="inline-flex flex-wrap items-center gap-x-2">
      <span aria-hidden="true" className="relative flex h-2 w-18 shrink-0 gap-0.5 overflow-hidden rounded-full">
        {Array.from({ length: days }, (_, day) => (
          <span key={day} className="h-full flex-1 bg-muted" />
        ))}
        <span className="absolute inset-y-0 rounded-full bg-muted-foreground" style={{ left: `${from}%`, right: `${100 - to}%` }} />
      </span>
      <span>{text}</span>
      {row.outsideSpan && <span className="font-semibold">{translate('overview-outside-span')}</span>}
    </span>
  )
}
