import { useQueryClient } from '@tanstack/react-query'
import { Link, Navigate, useParams } from '@tanstack/react-router'
import { useState } from 'react'
import { answerForMember, getOverviewQueryKey, resend, revoke, useGetGathering, useOverview } from '@/api/generated/api'
import { Button } from '@/components/ui/button'
import { Page } from '@/layout/Page'
import { useTranslate } from '@/l10n/translate'
import { ResponseForm, type SaveResult } from '@/responses/ResponseForm'
import { StatusBadge } from '@/responses/StatusBadge'

/** One invitation as organizers manage it: the response on the member's behalf and the personal link. */
export function InvitationPage() {
  const translate = useTranslate()
  const queryClient = useQueryClient()
  const { gatheringId, invitationId } = useParams({ from: '/organizer/gatherings/$gatheringId/invitations/$invitationId' })
  const gathering = useGetGathering(Number(gatheringId))
  const overview = useOverview(Number(gatheringId))
  const [notice, setNotice] = useState('')
  if (gathering.data?.status === 401 || overview.data?.status === 401) return <Navigate to="/sign-in" />
  if (gathering.data?.status !== 200 || overview.data?.status !== 200) return null
  const row = overview.data.data.rows.find((candidate) => String(candidate.invitationId) === invitationId)
  if (!row) return <Navigate to="/gatherings/$gatheringId" params={{ gatheringId }} />
  const { firstDay, lastDay, cancelled, ended } = gathering.data.data
  const name = `${row.firstName} ${row.lastName}`
  const refresh = () => queryClient.invalidateQueries({ queryKey: getOverviewQueryKey(Number(gatheringId)) })

  return (
    <Page title={translate('invitation-page-title', { name })}>
      <p className="-mt-2 mb-4">
        <Link to="/gatherings/$gatheringId" params={{ gatheringId }} className="underline underline-offset-4">
          {gathering.data.data.title}
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
              onClick={async () => {
                await resend(row.invitationId)
                await refresh()
                setNotice(translate('invitation-resent', { name: row.firstName }))
              }}
            >
              {translate('invitation-resend')}
            </Button>
            {row.linkActive && (
              <Button
                variant="outline"
                onClick={async () => {
                  await revoke(row.invitationId)
                  await refresh()
                  setNotice(translate('invitation-revoked', { name: row.firstName }))
                }}
              >
                {translate('invitation-revoke')}
              </Button>
            )}
          </div>
          <p role="status" className="mt-3 text-sm">
            {notice}
          </p>
        </section>
      )}
    </Page>
  )
}
