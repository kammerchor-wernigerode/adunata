import { createFileRoute } from '@tanstack/react-router'
import { useFragmentSecret } from '@/hooks/useFragmentSecret'
import { ConfirmSignIn } from './-components/ConfirmSignIn'
import { RequestSignIn } from './-components/RequestSignIn'

export const Route = createFileRoute('/sign-in/')({ component: SignInPage })

function SignInPage() {
  const [secret, setSecret] = useFragmentSecret()
  return secret ? <ConfirmSignIn key={secret} secret={secret} onRequestNew={() => setSecret('')} /> : <RequestSignIn />
}
