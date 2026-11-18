import { createFileRoute } from '@tanstack/react-router'
import { useGetGathering } from '@/api/generated/api'
import { PageTitle } from '@/components/PageTitle'
import { useTranslate } from '@/l10n/translate'
import { GatheringForm } from '../../-components/GatheringForm'

export const Route = createFileRoute('/_organizer/gatherings/$gatheringId/edit/')({ component: EditGatheringPage })

function EditGatheringPage() {
  const translate = useTranslate()
  const { gatheringId } = Route.useParams()
  const gathering = useGetGathering(Number(gatheringId))
  if (gathering.data?.status !== 200) return null
  return (
    <>
      <PageTitle title={translate('gathering-edit-title', { title: gathering.data.data.title })} />
      <GatheringForm gathering={gathering.data.data} />
    </>
  )
}
