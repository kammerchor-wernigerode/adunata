import { Check, Circle, CircleQuestionMark, X, type LucideIcon } from 'lucide-react'
import { useTranslate } from '@/l10n/translate'
import { cn } from '@/lib/utils'

export type ResponseStatus = 'pending' | 'accepted' | 'tentative' | 'declined'

const looks: Record<ResponseStatus, { icon: LucideIcon; className: string; iconClassName: string }> = {
  accepted: { icon: Check, className: 'bg-primary/15 text-foreground', iconClassName: 'text-primary' },
  tentative: { icon: CircleQuestionMark, className: 'bg-secondary text-foreground', iconClassName: 'text-foreground' },
  declined: { icon: X, className: 'bg-muted text-muted-foreground', iconClassName: '' },
  pending: { icon: Circle, className: 'text-muted-foreground ring-1 ring-border ring-inset', iconClassName: '' },
}

/** Shows a status as text with an icon. The icon is decoration, so a screen reader reads the text alone. */
export function StatusBadge({ status }: { status: ResponseStatus }) {
  const translate = useTranslate()
  const { icon: Icon, className, iconClassName } = looks[status]
  return (
    <span className={cn('inline-flex items-center gap-1 rounded-full py-0.5 pr-2.5 pl-2 text-sm font-semibold', className)}>
      <Icon aria-hidden="true" strokeWidth={3} className={cn('size-3.5', iconClassName)} />
      {translate(`status-${status}`)}
    </span>
  )
}
