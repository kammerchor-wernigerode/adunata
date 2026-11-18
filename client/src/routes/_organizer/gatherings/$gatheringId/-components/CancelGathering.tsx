import { useState } from 'react'
import { cancel } from '@/api/generated/api'
import type { Gathering } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import { useSubmission } from '@/hooks/useSubmission'
import { useTranslate } from '@/l10n/translate'

export function CancelGathering({ gathering, onCancelled }: { gathering: Gathering; onCancelled: () => Promise<void> }) {
  const translate = useTranslate()
  const [asking, setAsking] = useState(false)
  const { pending, failed, submit } = useSubmission()
  return (
    <div className="mt-10 border-t border-border pt-6">
      {asking ? (
        <div className="flex flex-col items-start gap-3">
          <p>{translate('gathering-cancel-confirm', { title: gathering.title })}</p>
          <div className="flex flex-wrap gap-3">
            <Button
              variant="destructive"
              disabled={pending}
              onClick={() =>
                submit(async () => {
                  if ((await cancel(gathering.id)).status !== 204) return false
                  await onCancelled()
                  return true
                })
              }
            >
              {translate('gathering-cancel-yes')}
            </Button>
            <Button variant="outline" onClick={() => setAsking(false)}>
              {translate('member-cancel')}
            </Button>
          </div>
          <p role="alert">{failed ? translate('error-unexpected') : ''}</p>
        </div>
      ) : (
        <Button variant="outline" onClick={() => setAsking(true)}>
          {translate('gathering-cancel')}
        </Button>
      )}
    </div>
  )
}
