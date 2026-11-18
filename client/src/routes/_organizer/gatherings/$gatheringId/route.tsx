import { createFileRoute, redirect } from '@tanstack/react-router'
import { getGetGatheringQueryOptions } from '@/api/generated/api'
import { dataOf } from '@/lib/dataOf'

/** Loads the gathering for every page below it, leaves for the list when it does not exist and fails on any other unexpected answer. */
export const Route = createFileRoute('/_organizer/gatherings/$gatheringId')({
  loader: async ({ context: { queryClient }, params: { gatheringId } }) => {
    const gathering = await queryClient.query(getGetGatheringQueryOptions(Number(gatheringId)))
    if (gathering.status === 404) throw redirect({ to: '/gatherings' })
    dataOf(gathering)
  },
})
