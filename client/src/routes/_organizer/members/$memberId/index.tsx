import { createFileRoute, Navigate, redirect } from '@tanstack/react-router'
import { getListMembersQueryOptions, useListMembers } from '@/api/generated/api'
import { PageTitle } from '@/components/PageTitle'
import { useTranslate } from '@/l10n/translate'
import { dataOf } from '@/lib/dataOf'
import { MemberForm } from '../-components/MemberForm'

export const Route = createFileRoute('/_organizer/members/$memberId/')({
  loader: async ({ context: { queryClient }, params: { memberId } }) => {
    const members = await queryClient.query(getListMembersQueryOptions())
    const list = dataOf(members)
    if (list && !list.some((member) => String(member.id) === memberId)) throw redirect({ to: '/members' })
  },
  component: EditMemberPage,
})

function EditMemberPage() {
  const translate = useTranslate()
  const { memberId } = Route.useParams()
  const members = useListMembers()
  const list = dataOf(members.data)
  if (!list) return null
  const member = list.find((candidate) => String(candidate.id) === memberId)
  if (!member) return <Navigate to="/members" />
  return (
    <>
      <PageTitle title={translate('member-edit-title', { name: `${member.firstName} ${member.lastName}` })} />
      <MemberForm member={member} />
    </>
  )
}
