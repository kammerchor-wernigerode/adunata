import { createFileRoute, redirect } from '@tanstack/react-router'
import { getGetGatheringQueryOptions } from '@/api/generated/api'

/** Loads the gathering for every page below it and leaves for the list when it does not exist. */
export const Route = createFileRoute('/_organizer/gatherings/$gatheringId')({
  loader: async ({ context: { queryClient }, params: { gatheringId } }) => {
    const gathering = await queryClient.ensureQueryData(getGetGatheringQueryOptions(Number(gatheringId)))
    if (gathering.status === 404) throw redirect({ to: '/gatherings' })
  },
})
