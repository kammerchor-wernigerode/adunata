import type { QueryClient } from '@tanstack/react-query'
import { createRouter, type RouterHistory } from '@tanstack/react-router'
import { RouteError } from '@/components/RouteError'
import { onExpiredSession } from '@/lib/expiredSession'
import { routeTree } from './routeTree.gen'

export type RouterContext = { queryClient: QueryClient }

export function createAppRouter(queryClient: QueryClient, history?: RouterHistory) {
  const router = createRouter({ routeTree, history, context: { queryClient }, defaultErrorComponent: RouteError })
  onExpiredSession(queryClient, () => void router.navigate({ to: '/sign-in' }))
  return router
}

declare module '@tanstack/react-router' {
  // oxlint-disable-next-line typescript/consistent-type-definitions -- module augmentation needs an interface
  interface Register {
    router: ReturnType<typeof createAppRouter>
  }
}
