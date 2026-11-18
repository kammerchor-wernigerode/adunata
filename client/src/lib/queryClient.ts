import { QueryClient, type DefaultOptions } from '@tanstack/react-query'

/** Data stays fresh briefly, so a page does not fetch again what its loader fetched a moment before. Saving invalidates what it changes. */
export function createQueryClient(queries: DefaultOptions['queries'] = {}) {
  return new QueryClient({ defaultOptions: { queries: { staleTime: 10_000, ...queries } } })
}
