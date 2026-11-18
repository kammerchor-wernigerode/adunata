import { createFileRoute, Link } from '@tanstack/react-router'
import { getListGatheringsQueryOptions, useListGatherings } from '@/api/generated/api'
import { PageTitle } from '@/components/PageTitle'
import { Button } from '@/components/ui/button'
import { useTranslate } from '@/l10n/translate'
import { GatheringSection } from './-components/GatheringSection'

export const Route = createFileRoute('/_organizer/gatherings/')({
  loader: ({ context: { queryClient } }) => queryClient.ensureQueryData(getListGatheringsQueryOptions()),
  component: GatheringListPage,
})

function GatheringListPage() {
  const translate = useTranslate()
  const gatherings = useListGatherings()
  const list = gatherings.data?.status === 200 ? gatherings.data.data : null

  return (
    <>
      <PageTitle title={translate('gatherings-title')} />
      <Button asChild className="mb-6">
        <Link to="/gatherings/new">{translate('gatherings-create')}</Link>
      </Button>
      {list && (
        <>
          <GatheringSection title={translate('gatherings-upcoming')} gatherings={list.upcoming} />
          <GatheringSection title={translate('gatherings-past')} gatherings={list.past} />
        </>
      )}
    </>
  )
}
