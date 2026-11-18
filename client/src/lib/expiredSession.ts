import { hashKey, type QueryClient } from '@tanstack/react-query'
import { getCurrentSessionQueryKey } from '@/api/generated/api'

/** Whether a response of the API says that the session is missing or has ended. */
export function isUnauthorized(response: unknown): boolean {
  return typeof response === 'object' && response !== null && 'status' in response && response.status === 401
}

/** Calls `signIn` whenever a query other than the session itself answers 401, such as a refetch after the session ended while a page was open. */
export function onExpiredSession(queryClient: QueryClient, signIn: () => void): () => void {
  const sessionHash = hashKey(getCurrentSessionQueryKey())
  return queryClient.getQueryCache().subscribe((event) => {
    if (event.type !== 'updated' || event.action.type !== 'success') return
    if (event.query.queryHash === sessionHash || !isUnauthorized(event.action.data)) return
    void queryClient.invalidateQueries({ queryKey: getCurrentSessionQueryKey() })
    signIn()
  })
}
