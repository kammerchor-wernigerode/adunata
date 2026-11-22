import { ChevronDownIcon } from 'lucide-react'
import { useState } from 'react'
import type { Gathering } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { useTranslate } from '@/l10n/translate'
import { GatheringList } from './GatheringList'
import { readYearChoices, storeYearChoices } from './pastYearsStorage'

type Year = { year: string; gatherings: Gathering[] }

/** Groups gatherings by the year of their first day, keeping the order of the list. */
function groupByYear(gatherings: Gathering[]): Year[] {
  const years: Year[] = []
  for (const gathering of gatherings) {
    const year = gathering.firstDay.slice(0, 4)
    const last = years.at(-1)
    if (last?.year === year) last.gatherings.push(gathering)
    else years.push({ year, gatherings: [gathering] })
  }
  return years
}

export function PastYears({ gatherings }: { gatherings: Gathering[] }) {
  const translate = useTranslate()
  const [choices, setChoices] = useState(readYearChoices)

  function choose(year: string, open: boolean) {
    const next = { ...choices, [year]: open }
    setChoices(next)
    storeYearChoices(next)
  }

  return groupByYear(gatherings).map(({ year, gatherings: ofYear }, index) => (
    <Collapsible key={year} open={choices[year] ?? index === 0} onOpenChange={(open) => choose(year, open)} className="mb-4">
      <h3 className="mb-2">
        <CollapsibleTrigger asChild>
          <Button variant="ghost" className="group px-2 font-heading text-base font-semibold">
            {year}{' '}
            <span className="font-sans text-sm font-normal text-muted-foreground">
              {translate('gatherings-count', { count: ofYear.length })}
            </span>
            <ChevronDownIcon aria-hidden className="transition-transform group-data-[state=open]:rotate-180" />
          </Button>
        </CollapsibleTrigger>
      </h3>
      <CollapsibleContent>
        <GatheringList gatherings={ofYear} />
      </CollapsibleContent>
    </Collapsible>
  ))
}
