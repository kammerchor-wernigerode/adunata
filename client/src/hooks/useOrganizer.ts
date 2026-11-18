import { getCurrentSessionQueryOptions, useCurrentSession } from '@/api/generated/api'

const sessionQuery = { query: { retry: false, staleTime: 60_000 } }

export function sessionQueryOptions() {
  return getCurrentSessionQueryOptions(sessionQuery)
}

/** The signed-in organizer, `null` without a valid session and `undefined` while it loads. */
export function useOrganizer() {
  const session = useCurrentSession(sessionQuery)
  if (session.isPending) return undefined
  return session.data?.status === 200 ? session.data.data : null
}
