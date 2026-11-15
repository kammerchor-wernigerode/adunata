import { Navigate, Outlet } from '@tanstack/react-router'
import { useOrganizer } from './session'

/** Shows organizer pages only with a valid session and sends everyone else to the sign-in page. */
export function RequireOrganizer() {
  const organizer = useOrganizer()
  if (organizer === undefined) return null
  if (organizer === null) return <Navigate to="/sign-in" />
  return <Outlet />
}
