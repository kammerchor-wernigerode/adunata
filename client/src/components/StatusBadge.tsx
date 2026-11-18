import { Check, Circle, CircleQuestionMark, X, type LucideIcon } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { useTranslate } from '@/l10n/translate'
import { cn } from '@/lib/utils'

export type ResponseStatus = 'pending' | 'accepted' | 'tentative' | 'declined'

const looks: Record<ResponseStatus, { icon: LucideIcon; className: string; iconClassName: string }> = {
  accepted: { icon: Check, className: 'border-transparent bg-primary/15 text-foreground', iconClassName: 'text-primary' },
  tentative: { icon: CircleQuestionMark, className: 'border-transparent bg-secondary text-foreground', iconClassName: 'text-foreground' },
  declined: { icon: X, className: 'border-transparent bg-muted text-muted-foreground', iconClassName: '' },
  pending: { icon: Circle, className: 'text-muted-foreground', iconClassName: '' },
}

/** Shows a status as text with an icon. The icon is decoration, so a screen reader reads the text alone. */
export function StatusBadge({ status }: { status: ResponseStatus }) {
  const translate = useTranslate()
  const { icon: Icon, className, iconClassName } = looks[status]
  return (
    <Badge variant="outline" className={cn('h-auto py-0.5 pr-2.5 pl-2 text-sm font-semibold', className)}>
      <Icon aria-hidden="true" strokeWidth={3} className={cn('size-3.5!', iconClassName)} />
      {translate(`status-${status}`)}
    </Badge>
  )
}
