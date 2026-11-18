import { useEffect, useState } from 'react'
import { deleteMember } from '@/api/generated/api'
import type { Member } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import { useTranslate } from '@/l10n/translate'
import { memberErrorMessage } from './memberErrorMessage'

export function DeleteMember({
  member,
  onDeleted,
  onRefused,
}: {
  member: Member
  onDeleted: () => Promise<void>
  onRefused: (message: string) => void
}) {
  const translate = useTranslate()
  const [asking, setAsking] = useState(false)
  const name = `${member.firstName} ${member.lastName}`
  useEffect(() => {
    if (asking) document.getElementById('delete-member-question')?.focus()
  }, [asking])

  async function remove() {
    const result = await deleteMember(member.id)
    if (result.status === 204) return onDeleted()
    setAsking(false)
    const [reason] = result.status === 422 ? result.data.errors : []
    onRefused(reason ? memberErrorMessage(translate, reason) : translate('error-unexpected'))
  }

  return (
    <section className="mt-10 border-t border-border pt-6">
      {asking ? (
        <div className="flex flex-col items-start gap-3">
          <p id="delete-member-question" tabIndex={-1}>
            {translate('member-delete-confirm', { name })}
          </p>
          <div className="flex flex-wrap gap-3">
            <Button type="button" variant="destructive" onClick={remove}>
              {translate('member-delete-yes')}
            </Button>
            <Button type="button" variant="outline" onClick={() => setAsking(false)}>
              {translate('member-cancel')}
            </Button>
          </div>
        </div>
      ) : (
        <Button type="button" variant="outline" onClick={() => setAsking(true)}>
          {translate('member-delete')}
        </Button>
      )}
    </section>
  )
}
