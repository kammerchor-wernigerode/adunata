import { Page } from '@/layout/Page'
import { useTranslate } from '@/l10n/translate'
import { DayStrip } from './DayStrip'
import { useInvitationPage, usePersonalLinkSecret } from './personalLink'

/** The page a member reaches through the personal link of an invitation. */
export function ResponsePage() {
  const translate = useTranslate()
  const secret = usePersonalLinkSecret()
  const page = useInvitationPage(secret)

  if (secret === '' || page.data?.status === 404) {
    return (
      <Page title={translate('invitation-invalid-title')}>
        <p>{translate('invitation-invalid')}</p>
      </Page>
    )
  }
  if (page.data?.status !== 200) return null
  const { gathering, state } = page.data.data

  return (
    <Page title={gathering.title}>
      <p className="-mt-2 text-muted-foreground">{gathering.location}</p>
      {gathering.description && <p className="mt-2 max-w-prose whitespace-pre-line">{gathering.description}</p>}
      <DayStrip firstDay={gathering.firstDay} lastDay={gathering.lastDay} />
      {state !== 'open' && (
        <p role="status" className="mt-6 font-semibold">
          {translate(`invitation-state-${state}`)}
        </p>
      )}
    </Page>
  )
}
