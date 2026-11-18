import { createFileRoute } from '@tanstack/react-router'
import { PageTitle } from '@/components/PageTitle'
import { useTranslate } from '@/l10n/translate'
import { GatheringForm } from '../-components/GatheringForm'

export const Route = createFileRoute('/_organizer/gatherings/new/')({ component: NewGatheringPage })

function NewGatheringPage() {
  const translate = useTranslate()
  return (
    <>
      <PageTitle title={translate('gathering-new-title')} />
      <GatheringForm />
    </>
  )
}
