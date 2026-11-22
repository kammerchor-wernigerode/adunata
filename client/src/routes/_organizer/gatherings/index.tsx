import { createFileRoute, Link } from '@tanstack/react-router'
import { getListGatheringsQueryOptions, useListGatherings } from '@/api/generated/api'
import { PageTitle } from '@/components/PageTitle'
import { Button } from '@/components/ui/button'
import { useTranslate } from '@/l10n/translate'
import { dataOf } from '@/lib/dataOf'
import { GatheringList } from './-components/GatheringList'
import { GatheringSection } from './-components/GatheringSection'
import { PastYears } from './-components/PastYears'

export const Route = createFileRoute('/_organizer/gatherings/')({
  loader: ({ context: { queryClient } }) => queryClient.query(getListGatheringsQueryOptions()),
  component: GatheringListPage,
})

function GatheringListPage() {
  const translate = useTranslate()
  const gatherings = useListGatherings()
  const list = dataOf(gatherings.data)

  return (
    <>
      <PageTitle
        title={translate('gatherings-title')}
        action={
          <Button asChild>
            <Link to="/gatherings/new">{translate('gatherings-create')}</Link>
          </Button>
        }
      />
      {list && (
        <>
          <GatheringSection title={translate('gatherings-upcoming')} gatherings={list.upcoming}>
            <GatheringList gatherings={list.upcoming} />
          </GatheringSection>
          <GatheringSection title={translate('gatherings-past')} gatherings={list.past}>
            <PastYears gatherings={list.past} />
          </GatheringSection>
        </>
      )}
    </>
  )
}
