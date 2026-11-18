import { createFileRoute, redirect } from '@tanstack/react-router'
import { getListMembersQueryOptions, useListMembers } from '@/api/generated/api'
import { PageTitle } from '@/components/PageTitle'
import { useTranslate } from '@/l10n/translate'
import { MemberForm } from '../-components/MemberForm'

export const Route = createFileRoute('/_organizer/members/$memberId/')({
  loader: async ({ context: { queryClient }, params: { memberId } }) => {
    const members = await queryClient.ensureQueryData(getListMembersQueryOptions())
    if (members.status === 200 && !members.data.some((member) => String(member.id) === memberId)) throw redirect({ to: '/members' })
  },
  component: EditMemberPage,
})

function EditMemberPage() {
  const translate = useTranslate()
  const { memberId } = Route.useParams()
  const members = useListMembers()
  const member = members.data?.status === 200 ? members.data.data.find((candidate) => String(candidate.id) === memberId) : undefined
  if (!member) return null
  return (
    <>
      <PageTitle title={translate('member-edit-title', { name: `${member.firstName} ${member.lastName}` })} />
      <MemberForm member={member} />
    </>
  )
}
