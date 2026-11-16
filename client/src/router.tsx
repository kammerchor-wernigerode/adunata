import { createRootRoute, createRoute, createRouter } from '@tanstack/react-router'
import { RequireOrganizer } from '@/access/RequireOrganizer'
import { SignInPage } from '@/access/SignInPage'
import { EditGatheringPage, NewGatheringPage } from '@/gatherings/GatheringFormPage'
import { GatheringListPage } from '@/gatherings/GatheringListPage'
import { GatheringPage } from '@/gatherings/GatheringPage'
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

const gatheringListRoute = createRoute({
  getParentRoute: () => organizerRoute,
  path: '/gatherings',
  component: GatheringListPage,
})

const newGatheringRoute = createRoute({
  getParentRoute: () => organizerRoute,
  path: '/gatherings/new',
  component: NewGatheringPage,
})

const gatheringRoute = createRoute({
  getParentRoute: () => organizerRoute,
  path: '/gatherings/$gatheringId',
  component: GatheringPage,
})

const editGatheringRoute = createRoute({
  getParentRoute: () => organizerRoute,
  path: '/gatherings/$gatheringId/edit',
  component: EditGatheringPage,
})

export const routeTree = rootRoute.addChildren([
  spikeRoute,
  signInRoute,
  organizerRoute.addChildren([
    gatheringListRoute,
    newGatheringRoute,
    gatheringRoute,
    editGatheringRoute,
    memberListRoute,
    newMemberRoute,
    editMemberRoute,
  ]),
])

export const router = createRouter({ routeTree })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
