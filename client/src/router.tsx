import { createRootRoute, createRoute, createRouter, Outlet } from '@tanstack/react-router'
import { LanguageSwitcher } from '@/l10n/LanguageSwitcher'
import { SpikeForm } from '@/spike/SpikeForm'

const rootRoute = createRootRoute({
  component: () => (
    <main className="p-6">
      <header className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Adunata</h1>
        <LanguageSwitcher />
      </header>
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
