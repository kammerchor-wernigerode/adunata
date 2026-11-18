import { useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate } from '@tanstack/react-router'
import { signOut } from '@/api/generated/api'
import { useOrganizer } from '@/hooks/useOrganizer'
import { useTranslate } from '@/l10n/translate'

const item =
  'rounded-sm underline-offset-4 decoration-2 outline-none hover:underline focus-visible:ring-3 focus-visible:ring-primary-foreground aria-[current=page]:underline'

export function OrganizerNavigation() {
  const translate = useTranslate()
  const organizer = useOrganizer()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  if (!organizer) return null

  async function leave() {
    await signOut()
    queryClient.clear()
    await navigate({ to: '/sign-in' })
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
          <button type="button" onClick={leave} className={item}>
            {translate('nav-sign-out')}
          </button>
        </li>
      </ul>
    </nav>
  )
}
