import { redirect } from '@tanstack/react-router'

/** Sends an organizer whose session ended while the client was open to the sign-in page. */
export function requireSession(response: { status: number }) {
  if (response.status === 401) throw redirect({ to: '/sign-in' })
}
