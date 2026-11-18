import { useQueryClient } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { answer } from '@/api/generated/api'
import { PageTitle } from '@/components/PageTitle'
import { ResponseForm, type SaveResult } from '@/components/ResponseForm'
import { formatDay } from '@/l10n/dates'
import { useLocale, useTranslate } from '@/l10n/translate'
import { dataOf } from '@/lib/dataOf'
import { Attendees } from './-components/Attendees'
import { DayStrip } from './-components/DayStrip'
import { authorization, invitationQueryKey, useInvitationPage, usePersonalLinkSecret } from './-personalLink'

export const Route = createFileRoute('/invitation/')({ component: ResponsePage })

/** The page a member reaches through the personal link of an invitation. */
function ResponsePage() {
  const translate = useTranslate()
  const { locale } = useLocale()
  const queryClient = useQueryClient()
  const secret = usePersonalLinkSecret()
  const page = useInvitationPage(secret)

  if (secret === '' || page.data?.status === 404) {
    return (
      <>
        <PageTitle title={translate('invitation-invalid-title')} />
        <p>{translate('invitation-invalid')}</p>
      </>
    )
  }
  if (page.isError) throw page.error
  const data = dataOf(page.data)
  if (!data) return null
  const { gathering, state, firstName, response, attendees } = data

  return (
    <>
      <PageTitle title={gathering.title} />
      <p className="-mt-2 text-muted-foreground">{gathering.location}</p>
      {gathering.description && <p className="mt-2 max-w-prose whitespace-pre-line">{gathering.description}</p>}
      <DayStrip firstDay={gathering.firstDay} lastDay={gathering.lastDay} />
      <div className="mt-6 max-w-xl">
        {state === 'open' ? (
          <>
            <p className="mb-4 text-sm text-muted-foreground">{translate('response-visibility')}</p>
            <ResponseForm
              key={secret}
              question={translate('response-question', { firstName })}
              firstDay={gathering.firstDay}
              lastDay={gathering.lastDay}
              initial={response}
              saveNotice={translate('response-saved', { day: formatDay(gathering.firstDay, locale) })}
              onSave={async (values): Promise<SaveResult> => {
                const result = await answer(values, authorization(secret))
                if (result.status === 200) {
                  await queryClient.invalidateQueries({ queryKey: invitationQueryKey(secret) })
                  return { status: 'saved', values: result.data }
                }
                if (result.status === 422) return { status: 'invalid', errors: result.data.errors }
                return { status: 'failed' }
              }}
            />
          </>
        ) : (
          <>
            <p role="status" className="font-semibold">
              {translate(`invitation-state-${state}`)}
            </p>
            {response && (
              <p className="mt-2">
                {translate('response-yours')}: {translate(`response-status-${response.status}`)}
              </p>
            )}
          </>
        )}
      </div>
      {attendees && <Attendees attendees={attendees} />}
    </>
  )
}
