import { createRootRoute, createRoute, createRouter, Navigate } from '@tanstack/react-router'
import { RequireOrganizer } from '@/access/RequireOrganizer'
import { SignInPage } from '@/access/SignInPage'
import { EditGatheringPage, NewGatheringPage } from '@/gatherings/GatheringFormPage'
import { GatheringListPage } from '@/gatherings/GatheringListPage'
import { GatheringPage } from '@/gatherings/GatheringPage'
import { InvitationPage } from '@/gatherings/InvitationPage'
import { Frame } from '@/layout/Frame'
import { EditMemberPage, NewMemberPage } from '@/members/MemberFormPage'
import { MemberListPage } from '@/members/MemberListPage'
import { ResponsePage } from '@/responses/ResponsePage'

const rootRoute = createRootRoute({ component: Frame })

const startRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: () => <Navigate to="/gatherings" />,
})

const signInRoute = createRoute({ getParentRoute: () => rootRoute, path: '/sign-in', component: SignInPage })

const responseRoute = createRoute({ getParentRoute: () => rootRoute, path: '/invitation', component: ResponsePage })

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

const invitationRoute = createRoute({
  getParentRoute: () => organizerRoute,
  path: '/gatherings/$gatheringId/invitations/$invitationId',
  component: InvitationPage,
})

export const routeTree = rootRoute.addChildren([
  startRoute,
  signInRoute,
  responseRoute,
  organizerRoute.addChildren([
    gatheringListRoute,
    newGatheringRoute,
    gatheringRoute,
    editGatheringRoute,
    invitationRoute,
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
