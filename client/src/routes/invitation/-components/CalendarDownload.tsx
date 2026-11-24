import { useState } from 'react'
import { calendarFile } from '@/api/generated/api'
import { Button } from '@/components/ui/button'
import { useTranslate } from '@/l10n/translate'
import { authorization } from '../-personalLink'

/** Saves the calendar file of the gathering as `gathering.ics`. The secret travels only in the `Authorization` header. */
export function CalendarDownload({ secret }: { secret: string }) {
  const translate = useTranslate()
  const [failed, setFailed] = useState(false)

  async function save() {
    setFailed(false)
    const result = await calendarFile(authorization(secret)).catch(() => null)
    if (result?.status !== 200) {
      setFailed(true)
      return
    }
    const url = URL.createObjectURL(new Blob([result.data], { type: 'text/calendar;charset=utf-8' }))
    const anchor = document.createElement('a')
    anchor.href = url
    anchor.download = 'gathering.ics'
    anchor.click()
    URL.revokeObjectURL(url)
  }

  return (
    <div className="mt-6 max-w-xl">
      <Button type="button" variant="outline" onClick={() => void save()}>
        {translate('response-calendar')}
      </Button>
      <p className="mt-2 text-sm text-muted-foreground">{translate('response-calendar-hint')}</p>
      <p role="alert" className="mt-2 text-sm text-destructive empty:mt-0">
        {failed ? translate('response-calendar-failed') : ''}
      </p>
    </div>
  )
}
