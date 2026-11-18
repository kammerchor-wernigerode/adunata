import { Link } from '@tanstack/react-router'
import type { Gathering, OverviewRow } from '@/api/generated/model'
import { StatusBadge } from '@/components/StatusBadge'
import { useTranslate, type Translate } from '@/l10n/translate'
import { MailNotice } from './MailNotice'
import { Period } from './Period'

const cell = 'px-3 py-2 align-top @max-[40rem]:flex @max-[40rem]:gap-3 @max-[40rem]:px-0 @max-[40rem]:py-0.5'
const label = 'sr-only @max-[40rem]:not-sr-only @max-[40rem]:min-w-24 @max-[40rem]:text-sm @max-[40rem]:text-muted-foreground'

function beds(translate: Translate, row: OverviewRow): string {
  if (row.beds === 'need') return translate('overview-beds-need', { count: row.bedCount ?? 0 })
  if (row.beds === 'offer') return translate('overview-beds-offer', { count: row.bedCount ?? 0 })
  return '–'
}

export function OverviewTable({ gathering, rows }: { gathering: Gathering; rows: OverviewRow[] }) {
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
                  <Link
                    to="/gatherings/$gatheringId/invitations/$invitationId"
                    params={{ gatheringId: String(gathering.id), invitationId: String(row.invitationId) }}
                    className="text-foreground underline underline-offset-4"
                  >
                    {row.firstName} {row.lastName}
                  </Link>
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
