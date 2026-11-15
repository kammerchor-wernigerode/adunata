import { useCurrentSession } from '@/api/generated/api'

/** The signed-in organizer, `null` without a valid session and `undefined` while it loads. */
export function useOrganizer() {
  const session = useCurrentSession({ query: { retry: false, staleTime: 60_000 } })
  if (session.isPending) return undefined
  return session.data?.status === 200 ? session.data.data : null
}
