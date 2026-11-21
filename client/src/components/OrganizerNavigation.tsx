import { Link } from '@tanstack/react-router'
import { useTranslate } from '@/l10n/translate'

const item =
  'rounded-sm underline-offset-4 decoration-2 outline-none hover:underline focus-visible:ring-3 focus-visible:ring-primary-foreground aria-[current=page]:underline'

export function OrganizerNavigation() {
  const translate = useTranslate()
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
      </ul>
    </nav>
  )
}
