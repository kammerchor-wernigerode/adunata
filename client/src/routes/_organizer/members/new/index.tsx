import { createFileRoute } from '@tanstack/react-router'
import { PageTitle } from '@/components/PageTitle'
import { useTranslate } from '@/l10n/translate'
import { MemberForm } from '../-components/MemberForm'

export const Route = createFileRoute('/_organizer/members/new/')({ component: NewMemberPage })

function NewMemberPage() {
  const translate = useTranslate()
  return (
    <>
      <PageTitle title={translate('member-new-title')} />
      <MemberForm />
    </>
  )
}
