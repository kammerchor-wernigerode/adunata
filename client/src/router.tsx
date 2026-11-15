import { createRootRoute, createRoute, createRouter } from '@tanstack/react-router'
import { RequireOrganizer } from '@/access/RequireOrganizer'
import { SignInPage } from '@/access/SignInPage'
import { Frame } from '@/layout/Frame'
import { EditMemberPage, NewMemberPage } from '@/members/MemberFormPage'
import { MemberListPage } from '@/members/MemberListPage'
import { SpikePage } from '@/spike/SpikePage'

const rootRoute = createRootRoute({ component: Frame })

const spikeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: SpikePage,
})

const signInRoute = createRoute({ getParentRoute: () => rootRoute, path: '/sign-in', component: SignInPage })

const organizerRoute = createRoute({ getParentRoute: () => rootRoute, id: 'organizer', component: RequireOrganizer })

const memberListRoute = createRoute({ getParentRoute: () => organizerRoute, path: '/members', component: MemberListPage })

const newMemberRoute = createRoute({ getParentRoute: () => organizerRoute, path: '/members/new', component: NewMemberPage })

const editMemberRoute = createRoute({
  getParentRoute: () => organizerRoute,
  path: '/members/$memberId',
  component: EditMemberPage,
})

export const routeTree = rootRoute.addChildren([
  spikeRoute,
  signInRoute,
  organizerRoute.addChildren([memberListRoute, newMemberRoute, editMemberRoute]),
])

export const router = createRouter({ routeTree })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
