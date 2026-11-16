import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import { invitationPage } from '@/api/generated/api'

/** The secret of the personal link. It lives in the fragment, which the browser never sends, and leaves the client only in the `Authorization` header. */
export function usePersonalLinkSecret(): string {
  const [secret] = useState(() => window.location.hash.slice(1))
  return secret
}

export function authorization(secret: string): RequestInit {
  return { headers: { Authorization: `Bearer ${secret}` } }
}

export function invitationQueryKey(secret: string) {
  return ['invitation', secret] as const
}

export function useInvitationPage(secret: string) {
  return useQuery({
    queryKey: invitationQueryKey(secret),
    queryFn: () => invitationPage(authorization(secret)),
    enabled: secret !== '',
    retry: false,
  })
}
