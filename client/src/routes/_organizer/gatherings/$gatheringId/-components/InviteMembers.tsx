import { useId, useState } from 'react'
import { invite, useListMembers } from '@/api/generated/api'
import type { Gathering, Invitation } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import { useSubmission } from '@/hooks/useSubmission'
import { dataOf } from '@/lib/dataOf'
import { Checkbox } from '@/components/ui/checkbox'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { Field, FieldLabel, FieldLegend, FieldSet } from '@/components/ui/field'
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
  const candidates = dataOf(members.data)?.filter((member) => member.active && !invitedIds.has(member.id))

  const { pending, failed, submit } = useSubmission()

  function send(memberIds?: number[]) {
    return submit(async () => {
      const result = await invite(gathering.id, memberIds ? { memberIds } : {})
      if (result.status !== 201) return false
      setChosen([])
      await onInvited(translate('invitations-sent', { count: result.data.count }))
      return true
    })
  }

  if (!candidates) return null
  if (candidates.length === 0) return <p className="text-muted-foreground">{translate('invitations-everyone-invited')}</p>

  return (
    <div className="flex flex-col items-start gap-3">
      <Button disabled={pending} onClick={() => send()}>{translate('invitations-invite-all', { count: candidates.length })}</Button>
      <Collapsible className="w-full">
        <CollapsibleTrigger asChild>
          <Button variant="link" className="px-0">
            {translate('invitations-choose')}
          </Button>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <FieldSet className="mt-3 gap-2">
            <FieldLegend className="sr-only">{translate('invitations-choose')}</FieldLegend>
            {candidates.map((member) => (
              <Field key={member.id} orientation="horizontal">
                <Checkbox
                  id={`${id}-${member.id}`}
                  checked={chosen.includes(member.id)}
                  onCheckedChange={(checked) =>
                    setChosen(checked === true ? [...chosen, member.id] : chosen.filter((other) => other !== member.id))
                  }
                />
                <FieldLabel htmlFor={`${id}-${member.id}`} className="font-normal">
                  {member.lastName}, {member.firstName}
                </FieldLabel>
              </Field>
            ))}
            <Button variant="outline" className="w-fit" disabled={pending || chosen.length === 0} onClick={() => send(chosen)}>
              {translate('invitations-invite-chosen', { count: chosen.length })}
            </Button>
          </FieldSet>
        </CollapsibleContent>
      </Collapsible>
      <p role="alert">{failed ? translate('error-unexpected') : ''}</p>
    </div>
  )
}
