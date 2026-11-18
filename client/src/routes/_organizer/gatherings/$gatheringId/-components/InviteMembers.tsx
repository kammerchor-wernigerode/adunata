import { useId, useState } from 'react'
import { invite, useListMembers } from '@/api/generated/api'
import type { Gathering, Invitation } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import { useTranslate } from '@/l10n/translate'

export function InviteMembers({
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
