import { createRootRoute, createRoute, createRouter, Outlet } from '@tanstack/react-router'
import { SpikeForm } from '@/spike/SpikeForm'

const rootRoute = createRootRoute({
  component: () => (
    <main className="p-6">
      <h1 className="mb-6 text-2xl font-semibold">Adunata</h1>
      <Outlet />
    </main>
  ),
})

const spikeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: SpikeForm,
})

export const router = createRouter({ routeTree: rootRoute.addChildren([spikeRoute]) })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
