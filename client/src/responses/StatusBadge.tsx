import { Check, Circle, CircleQuestionMark, X, type LucideIcon } from 'lucide-react'
import { useTranslate } from '@/l10n/translate'
import { cn } from '@/lib/utils'

export type ResponseStatus = 'pending' | 'accepted' | 'tentative' | 'declined'

const looks: Record<ResponseStatus, { icon: LucideIcon; className: string }> = {
  accepted: { icon: Check, className: 'border-primary bg-primary/10 text-foreground' },
  tentative: { icon: CircleQuestionMark, className: 'border-dashed border-primary text-foreground' },
  declined: { icon: X, className: 'border-input text-foreground' },
  pending: { icon: Circle, className: 'border-input text-muted-foreground' },
}

/** Shows a status as text with an icon. The icon is decoration, so a screen reader reads the text alone. */
export function StatusBadge({ status }: { status: ResponseStatus }) {
  const translate = useTranslate()
  const { icon: Icon, className } = looks[status]
  return (
    <span className={cn('inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-sm', className)}>
      <Icon aria-hidden="true" className="size-4" />
      {translate(`status-${status}`)}
    </span>
  )
}
