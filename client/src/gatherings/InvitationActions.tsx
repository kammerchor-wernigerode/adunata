import { useQueryClient } from '@tanstack/react-query'
import { useId, useState } from 'react'
import {
  cancel,
  getGetGatheringQueryKey,
  getListGatheringsQueryKey,
  getListInvitationsQueryKey,
  invite,
  resend,
  revoke,
  useListInvitations,
  useListMembers,
} from '@/api/generated/api'
import type { Gathering, Invitation } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import { useTranslate } from '@/l10n/translate'

/** Invites members to a gathering and manages the links of its invitations. */
export function InvitationActions({ gathering }: { gathering: Gathering }) {
  const translate = useTranslate()
  const queryClient = useQueryClient()
  const invitations = useListInvitations(gathering.id)
  const [notice, setNotice] = useState('')
  const open = !gathering.cancelled && !gathering.ended
  const list = invitations.data?.status === 200 ? invitations.data.data : []

  async function refresh(message: string) {
    await queryClient.invalidateQueries({ queryKey: getListInvitationsQueryKey(gathering.id) })
    setNotice(message)
  }

  return (
    <section aria-labelledby="invitations-heading" className="mt-8">
      <h2 id="invitations-heading" className="mb-3 font-heading text-lg font-semibold">
        {translate('invitations-title')}
      </h2>
      {open && <InviteMembers gathering={gathering} invited={list} onInvited={refresh} />}
      <p role="status" className="my-3 text-sm">
        {notice}
      </p>
      {list.length > 0 && (
        <ul className="divide-y divide-border overflow-hidden rounded-lg border border-border">
          {list.map((invitation) => (
            <li key={invitation.id} className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2 px-4 py-2">
              <div className="min-w-0">
                <span className="font-semibold">
                  {invitation.firstName} {invitation.lastName}
                </span>
                <span className="ml-3 text-sm text-muted-foreground">
                  {translate(invitation.linkActive ? 'invitation-link-active' : 'invitation-link-revoked')}
                  {invitation.mail ? `, ${translate(`invitation-mail-${invitation.mail}`)}` : ''}
                </span>
              </div>
              {open && (
                <div className="flex flex-wrap gap-2">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={async () => {
                      await resend(invitation.id)
                      await refresh(translate('invitation-resent', { name: invitation.firstName }))
                    }}
                  >
                    {translate('invitation-resend')}
                    <span className="sr-only">: {invitation.firstName} {invitation.lastName}</span>
                  </Button>
                  {invitation.linkActive && (
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={async () => {
                        await revoke(invitation.id)
                        await refresh(translate('invitation-revoked', { name: invitation.firstName }))
                      }}
                    >
                      {translate('invitation-revoke')}
                      <span className="sr-only">: {invitation.firstName} {invitation.lastName}</span>
                    </Button>
                  )}
                </div>
              )}
            </li>
          ))}
        </ul>
      )}
      {open && (
        <CancelGathering
          gathering={gathering}
          onCancelled={async () => {
            await queryClient.invalidateQueries({ queryKey: getGetGatheringQueryKey(gathering.id) })
            await queryClient.invalidateQueries({ queryKey: getListGatheringsQueryKey() })
          }}
        />
      )}
    </section>
  )
}

function InviteMembers({
  gathering,
  invited,
  onInvited,
}: {
  gathering: Gathering
  invited: Invitation[]
  onInvited: (message: string) => Promise<void>
}) {
  const id = useId()
  const translate = useTranslate()
  const members = useListMembers()
  const [chosen, setChosen] = useState<number[]>([])
  const invitedIds = new Set(invited.map((invitation) => invitation.memberId))
  const candidates =
    members.data?.status === 200 ? members.data.data.filter((member) => member.active && !invitedIds.has(member.id)) : []

  async function send(memberIds?: number[]) {
    const result = await invite(gathering.id, memberIds ? { memberIds } : {})
    setChosen([])
    if (result.status === 201) await onInvited(translate('invitations-sent', { count: result.data.count }))
  }

  if (candidates.length === 0) return <p className="text-muted-foreground">{translate('invitations-everyone-invited')}</p>

  return (
    <div className="flex flex-col items-start gap-3">
      <Button onClick={() => send()}>{translate('invitations-invite-all', { count: candidates.length })}</Button>
      <details className="w-full">
        <summary className="cursor-pointer underline underline-offset-4">{translate('invitations-choose')}</summary>
        <fieldset className="mt-3 flex flex-col gap-2">
          <legend className="sr-only">{translate('invitations-choose')}</legend>
          {candidates.map((member) => (
            <label key={member.id} htmlFor={`${id}-${member.id}`} className="flex items-center gap-2">
              <input
                id={`${id}-${member.id}`}
                type="checkbox"
                className="size-4 accent-primary"
                checked={chosen.includes(member.id)}
                onChange={(event) =>
                  setChosen(event.target.checked ? [...chosen, member.id] : chosen.filter((other) => other !== member.id))
                }
              />
              {member.lastName}, {member.firstName}
            </label>
          ))}
          <Button variant="outline" className="w-fit" disabled={chosen.length === 0} onClick={() => send(chosen)}>
            {translate('invitations-invite-chosen', { count: chosen.length })}
          </Button>
        </fieldset>
      </details>
    </div>
  )
}

function CancelGathering({ gathering, onCancelled }: { gathering: Gathering; onCancelled: () => Promise<void> }) {
  const translate = useTranslate()
  const [asking, setAsking] = useState(false)
  return (
    <div className="mt-10 border-t border-border pt-6">
      {asking ? (
        <div className="flex flex-col items-start gap-3">
          <p>{translate('gathering-cancel-confirm', { title: gathering.title })}</p>
          <div className="flex flex-wrap gap-3">
            <Button
              variant="destructive"
              onClick={async () => {
                await cancel(gathering.id)
                await onCancelled()
              }}
            >
              {translate('gathering-cancel-yes')}
            </Button>
            <Button variant="outline" onClick={() => setAsking(false)}>
              {translate('member-cancel')}
            </Button>
          </div>
        </div>
      ) : (
        <Button variant="outline" onClick={() => setAsking(true)}>
          {translate('gathering-cancel')}
        </Button>
      )}
    </div>
  )
}
