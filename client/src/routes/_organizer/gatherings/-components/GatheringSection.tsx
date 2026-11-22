import type { ReactNode } from 'react'
import type { Gathering } from '@/api/generated/model'
import { useTranslate } from '@/l10n/translate'

export function GatheringSection({
  title,
  gatherings,
  children,
}: {
  title: string
  gatherings: Gathering[]
  children: ReactNode
}) {
  const translate = useTranslate()
  return (
    <section className="mb-8">
      <h2 className="mb-3 font-heading text-lg font-semibold">{title}</h2>
      {gatherings.length === 0 ? <p className="text-muted-foreground">{translate('gatherings-none')}</p> : children}
    </section>
  )
}
