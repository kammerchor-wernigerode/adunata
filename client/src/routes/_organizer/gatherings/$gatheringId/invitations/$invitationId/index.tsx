import { useQueryClient } from '@tanstack/react-query'
import { createFileRoute, Link, Navigate, redirect } from '@tanstack/react-router'
import { useState } from 'react'
import {
  answerForMember,
  getOverviewQueryKey,
  getOverviewQueryOptions,
  resend,
  revoke,
  useGetGathering,
  useOverview,
} from '@/api/generated/api'
import { PageTitle } from '@/components/PageTitle'
import { ResponseForm, type SaveResult } from '@/components/ResponseForm'
import { StatusBadge } from '@/components/StatusBadge'
import { Button } from '@/components/ui/button'
import { useTranslate } from '@/l10n/translate'
import { useSubmission } from '@/hooks/useSubmission'
import { dataOf } from '@/lib/dataOf'

export const Route = createFileRoute('/_organizer/gatherings/$gatheringId/invitations/$invitationId/')({
  loader: async ({ context: { queryClient }, params: { gatheringId, invitationId } }) => {
    const overview = await queryClient.query(getOverviewQueryOptions(Number(gatheringId)))
    const rows = dataOf(overview)?.rows
    if (rows && !rows.some((row) => String(row.invitationId) === invitationId)) throw redirect({ to: '/gatherings/$gatheringId', params: { gatheringId } })
  },
  component: InvitationPage,
})

/** One invitation as organizers manage it: the response on the member's behalf and the personal link. */
function InvitationPage() {
  const translate = useTranslate()
  const queryClient = useQueryClient()
  const { gatheringId, invitationId } = Route.useParams()
  const gathering = useGetGathering(Number(gatheringId))
  const overview = useOverview(Number(gatheringId))
  const [notice, setNotice] = useState('')
  const { pending, failed, submit } = useSubmission()
  const gatheringData = dataOf(gathering.data)
  const overviewData = dataOf(overview.data)
  if (!gatheringData || !overviewData) return null
  const row = overviewData.rows.find((candidate) => String(candidate.invitationId) === invitationId)
  if (!row) return <Navigate to="/gatherings/$gatheringId" params={{ gatheringId }} />
  const { firstDay, lastDay, cancelled, ended } = gatheringData
  const name = `${row.firstName} ${row.lastName}`
  const refresh = () => queryClient.invalidateQueries({ queryKey: getOverviewQueryKey(Number(gatheringId)) })

  function act(action: () => Promise<boolean>, success: string) {
    setNotice('')
    return submit(async () => {
      if (!(await action())) return false
      await refresh()
      setNotice(success)
      return true
    })
  }

  return (
    <>
      <PageTitle title={translate('invitation-page-title', { name })} />
      <p className="-mt-2 mb-4">
        <Link to="/gatherings/$gatheringId" params={{ gatheringId }} className="underline underline-offset-4">
          {gatheringData.title}
        </Link>
      </p>
      <p className="mb-6 flex flex-wrap items-center gap-2">
        <StatusBadge status={row.status} />
        {row.changedBy && row.changedAt && (
          <span className="text-sm text-muted-foreground">
            {translate(`invitation-changed-by-${row.changedBy}`, { at: row.changedAt.replace('T', ' ') })}
          </span>
        )}
      </p>
      <div className="max-w-xl">
        <ResponseForm
          question={translate('response-question-for', { name })}
          firstDay={firstDay}
          lastDay={lastDay}
          initial={row.status === 'pending' ? null : { ...row, status: row.status }}
          saveNotice={translate('response-saved-for', { name })}
          onSave={async (values): Promise<SaveResult> => {
            const result = await answerForMember(row.invitationId, values)
            if (result.status === 200) {
              await refresh()
              return { status: 'saved', values: result.data }
            }
            if (result.status === 422) return { status: 'invalid', errors: result.data.errors }
            return { status: 'failed' }
          }}
        />
      </div>
      {!cancelled && !ended && (
        <section aria-labelledby="link-heading" className="mt-10 border-t border-border pt-6">
          <h2 id="link-heading" className="mb-2 font-heading text-lg font-semibold">
            {translate('invitation-link-title')}
          </h2>
          <p className="mb-3 text-sm text-muted-foreground">
            {translate(row.linkActive ? 'invitation-link-active' : 'invitation-link-revoked')}
            {row.mail ? `, ${translate(`invitation-mail-${row.mail}`)}` : ''}
          </p>
          <div className="flex flex-wrap gap-3">
            <Button
              variant="outline"
              disabled={pending}
              onClick={() =>
                act(async () => (await resend(row.invitationId)).status === 204, translate('invitation-resent', { name: row.firstName }))
              }
            >
              {translate('invitation-resend')}
            </Button>
            {row.linkActive && (
              <Button
                variant="outline"
                disabled={pending}
                onClick={() =>
                  act(async () => (await revoke(row.invitationId)).status === 204, translate('invitation-revoked', { name: row.firstName }))
                }
              >
                {translate('invitation-revoke')}
              </Button>
            )}
          </div>
          <p role="status" className="mt-3 text-sm">
            {notice}
          </p>
          <p role="alert" className="text-sm">
            {failed ? translate('error-unexpected') : ''}
          </p>
        </section>
      )}
    </>
  )
}
