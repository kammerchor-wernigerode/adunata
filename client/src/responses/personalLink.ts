import { useQuery } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import { invitationPage } from '@/api/generated/api'

/** The secret of a link in the fragment, which the browser never sends. It follows the fragment when someone opens another link in the same tab. */
export function useFragmentSecret(): [string, (secret: string) => void] {
  const [secret, setSecret] = useState(() => window.location.hash.slice(1))
  useEffect(() => {
    const follow = () => setSecret(window.location.hash.slice(1))
    window.addEventListener('hashchange', follow)
    return () => window.removeEventListener('hashchange', follow)
  }, [])
  return [secret, setSecret]
}

/** The secret of the personal link. It leaves the client only in the `Authorization` header. */
export function usePersonalLinkSecret(): string {
  return useFragmentSecret()[0]
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
