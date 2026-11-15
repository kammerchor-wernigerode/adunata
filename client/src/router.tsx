import { createRootRoute, createRoute, createRouter } from '@tanstack/react-router'
import { Frame } from '@/layout/Frame'
import { SpikePage } from '@/spike/SpikePage'

const rootRoute = createRootRoute({ component: Frame })

const spikeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: SpikePage,
})

export const router = createRouter({ routeTree: rootRoute.addChildren([spikeRoute]) })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
