import { createFileRoute, redirect } from '@tanstack/react-router'
import { sessionQueryOptions } from '@/hooks/useOrganizer'

/** Shows organizer pages only with a valid session and sends everyone else to the sign-in page. */
export const Route = createFileRoute('/_organizer')({
  beforeLoad: async ({ context: { queryClient } }) => {
    const session = await queryClient.query(sessionQueryOptions())
    if (session.status !== 200) throw redirect({ to: '/sign-in' })
  },
})
