import { useQueryClient } from '@tanstack/react-query'
import { useState } from 'react'
import { getOverviewQueryKey, resend } from '@/api/generated/api'
import type { Gathering, OverviewRow } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import { useTranslate } from '@/l10n/translate'

export function MailNotice({ row, gathering }: { row: OverviewRow; gathering: Gathering }) {
  const translate = useTranslate()
  const queryClient = useQueryClient()
  const [sent, setSent] = useState(false)
  if (row.mail !== 'failed' && row.linkActive) return null
  const open = !gathering.cancelled && !gathering.ended
  return (
    <span className="mt-1 flex flex-wrap items-center gap-2 text-sm font-normal text-muted-foreground">
      {row.mail === 'failed' ? translate('invitation-mail-failed') : translate('invitation-link-revoked')}
      {open && !sent && (
        <Button
          size="sm"
          variant="outline"
          onClick={async () => {
            await resend(row.invitationId)
            setSent(true)
            await queryClient.invalidateQueries({ queryKey: getOverviewQueryKey(gathering.id) })
          }}
        >
          {translate('invitation-resend')}
          <span className="sr-only">
            : {row.firstName} {row.lastName}
          </span>
        </Button>
      )}
    </span>
  )
}
