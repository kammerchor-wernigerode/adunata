import { createFileRoute, redirect } from '@tanstack/react-router'
import { sessionQueryOptions } from '@/hooks/useOrganizer'
import { dataOf } from '@/lib/dataOf'

/** Shows organizer pages only with a valid session and sends everyone else to the sign-in page. */
export const Route = createFileRoute('/_organizer')({
  beforeLoad: async ({ context: { queryClient } }) => {
    const session = await queryClient.query(sessionQueryOptions())
    if (!dataOf(session)) throw redirect({ to: '/sign-in' })
  },
})
