import { calendarFile } from '@/api/generated/api'
import { Button } from '@/components/ui/button'
import { useTranslate } from '@/l10n/translate'
import { authorization } from '../-personalLink'

/** Saves the calendar file of the gathering as `gathering.ics`. The secret travels only in the `Authorization` header. */
export function CalendarDownload({ secret }: { secret: string }) {
  const translate = useTranslate()

  async function save() {
    const result = await calendarFile(authorization(secret))
    if (result.status !== 200) return
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
    </div>
  )
}
