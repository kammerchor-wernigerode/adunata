import { hashKey, type QueryClient } from '@tanstack/react-query'
import { getCurrentSessionQueryKey } from '@/api/generated/api'
import { statusOf } from '@/lib/dataOf'

/** Calls `signIn` whenever the session ends while the client is open. A query other than the session that answers 401 shows it, and so does the session itself once it turns from 200 to 401. */
export function onExpiredSession(queryClient: QueryClient, signIn: () => void) {
  const sessionHash = hashKey(getCurrentSessionQueryKey())
  let signedIn = false
  queryClient.getQueryCache().subscribe((event) => {
    if (event.type !== 'updated' || event.action.type !== 'success') return
    const status = statusOf(event.action.data)
    if (event.query.queryHash === sessionHash) {
      const ended = signedIn && status === 401
      signedIn = status === 200
      if (ended) signIn()
      return
    }
    if (status !== 401) return
    void queryClient.invalidateQueries({ queryKey: getCurrentSessionQueryKey() })
    signIn()
  })
}
