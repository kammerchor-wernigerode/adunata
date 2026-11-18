import { useQuery } from '@tanstack/react-query'
import { invitationPage } from '@/api/generated/api'
import { useFragmentSecret } from '@/hooks/useFragmentSecret'

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
