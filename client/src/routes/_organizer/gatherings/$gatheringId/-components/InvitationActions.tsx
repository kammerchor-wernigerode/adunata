import { useQueryClient } from '@tanstack/react-query'
import { useState } from 'react'
import {
  getGetGatheringQueryKey,
  getListGatheringsQueryKey,
  getListInvitationsQueryKey,
  getOverviewQueryKey,
  useListInvitations,
} from '@/api/generated/api'
import type { Gathering } from '@/api/generated/model'
import { useTranslate } from '@/l10n/translate'
import { dataOf } from '@/lib/dataOf'
import { CancelGathering } from './CancelGathering'
import { InviteMembers } from './InviteMembers'

/** Invites members to a gathering and cancels it. */
export function InvitationActions({ gathering }: { gathering: Gathering }) {
  const translate = useTranslate()
  const queryClient = useQueryClient()
  const invitations = useListInvitations(gathering.id)
  const [notice, setNotice] = useState('')
  const open = !gathering.cancelled && !gathering.ended
  const list = dataOf(invitations.data)

  async function refresh(message: string) {
    await queryClient.invalidateQueries({ queryKey: getListInvitationsQueryKey(gathering.id) })
    await queryClient.invalidateQueries({ queryKey: getOverviewQueryKey(gathering.id) })
    setNotice(message)
  }

  return (
    <section aria-labelledby="invitations-heading" className="mt-8">
      <h2 id="invitations-heading" className="mb-3 font-heading text-lg font-semibold">
        {translate('invitations-title')}
      </h2>
      {open && list && <InviteMembers gathering={gathering} invited={list} onInvited={refresh} />}
      <p role="status" className="my-3 text-sm">
        {notice}
      </p>
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
