import { useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate } from '@tanstack/react-router'
import { getCurrentSessionQueryKey, signOut } from '@/api/generated/api'
import { Button } from '@/components/ui/button'
import { useOrganizer } from '@/hooks/useOrganizer'
import { useSubmission } from '@/hooks/useSubmission'
import { useTranslate } from '@/l10n/translate'
import { cn } from '@/lib/utils'

const item =
  'rounded-sm underline-offset-4 decoration-2 outline-none hover:underline focus-visible:ring-3 focus-visible:ring-primary-foreground aria-[current=page]:underline'

export function OrganizerNavigation() {
  const translate = useTranslate()
  const organizer = useOrganizer()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { pending, failed, submit } = useSubmission()
  if (!organizer) return null

  function leave() {
    return submit(async () => {
      const result: { status: number } = await signOut()
      if (result.status !== 204) return false
      await navigate({ to: '/sign-in' })
      await queryClient.resetQueries({ queryKey: getCurrentSessionQueryKey() })
      queryClient.removeQueries({ type: 'inactive' })
      return true
    })
  }

  return (
    <nav aria-label={translate('nav-label')}>
      <ul className="flex flex-wrap gap-4">
        <li>
          <Link to="/gatherings" className={item}>
            {translate('nav-gatherings')}
          </Link>
        </li>
        <li>
          <Link to="/members" className={item}>
            {translate('nav-members')}
          </Link>
        </li>
        <li>
          <Button variant="link" disabled={pending} onClick={leave} className={cn(item, 'h-auto p-0 text-base font-normal text-primary-foreground')}>
            {translate('nav-sign-out')}
          </Button>
          <span role="alert" className="ml-2">
            {failed ? translate('error-unexpected') : ''}
          </span>
        </li>
      </ul>
    </nav>
  )
}
