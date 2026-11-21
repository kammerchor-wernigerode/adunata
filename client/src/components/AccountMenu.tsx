import { useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate } from '@tanstack/react-router'
import { ChevronDownIcon } from 'lucide-react'
import { useRef, useState } from 'react'
import { getCurrentSessionQueryKey, signOut } from '@/api/generated/api'
import type { Member } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useSubmission } from '@/hooks/useSubmission'
import { isLocale, supportedLocales } from '@/l10n/bundle'
import { useLocale, useTranslate } from '@/l10n/translate'

type AccountMenuProps = { organizer: Member }

/** The signed-in organizer's menu with the own member entry, the language and sign-out. Names a failed sign-out next to its trigger, so the message outlasts the menu. */
export function AccountMenu({ organizer }: AccountMenuProps) {
  const translate = useTranslate()
  const { locale, setLocale } = useLocale()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { pending, failed, submit } = useSubmission()
  const [header, setHeader] = useState<HTMLDivElement | null>(null)
  const leavingPage = useRef(false)
  const name = `${organizer.firstName} ${organizer.lastName}`

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
    <div ref={setHeader} className="flex min-w-0 items-center gap-2">
      <span role="alert">{failed ? translate('error-unexpected') : ''}</span>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            aria-label={translate('account-menu-label', { name })}
            className="max-w-full min-w-0 shrink text-base font-normal focus-visible:ring-primary-foreground sm:max-w-xs"
          >
            <span className="truncate">{name}</span>
            <ChevronDownIcon aria-hidden />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          container={header}
          align="end"
          className="w-auto"
          onCloseAutoFocus={(event) => {
            if (leavingPage.current) event.preventDefault()
            leavingPage.current = false
          }}
        >
          <DropdownMenuItem asChild onSelect={() => (leavingPage.current = true)}>
            <Link to="/members/$memberId" params={{ memberId: String(organizer.id) }}>
              {translate('account-own-entry')}
            </Link>
          </DropdownMenuItem>
          <DropdownMenuLabel>{translate('language-label')}</DropdownMenuLabel>
          <DropdownMenuRadioGroup value={locale} onValueChange={(value) => isLocale(value) && setLocale(value)}>
            {supportedLocales.map((language) => (
              <DropdownMenuRadioItem key={language} value={language} lang={language}>
                {translate(`language-name-${language}`)}
              </DropdownMenuRadioItem>
            ))}
          </DropdownMenuRadioGroup>
          <DropdownMenuSeparator />
          <DropdownMenuItem disabled={pending} onSelect={leave}>
            {translate('nav-sign-out')}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  )
}
