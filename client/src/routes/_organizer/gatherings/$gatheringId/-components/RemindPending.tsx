import { useState } from 'react'
import { remind } from '@/api/generated/api'
import type { Gathering } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { useSubmission } from '@/hooks/useSubmission'
import { useTranslate } from '@/l10n/translate'

/** Asks for confirmation and then reminds every pending member with an active link. */
export function RemindPending({
  gathering,
  count,
  onReminded,
}: {
  gathering: Gathering
  count: number
  onReminded: (message: string) => Promise<void>
}) {
  const translate = useTranslate()
  const [open, setOpen] = useState(false)
  const { pending, failed, submit, reset } = useSubmission()

  function send() {
    return submit(async () => {
      const result = await remind(gathering.id)
      if (result.status !== 201) return false
      setOpen(false)
      await onReminded(translate('reminders-sent', { count: result.data.count }))
      return true
    })
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        setOpen(next)
        reset()
      }}
    >
      <DialogTrigger asChild>
        <Button variant="outline">{translate('overview-remind', { count })}</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{translate('overview-remind-title')}</DialogTitle>
          <DialogDescription>{translate('overview-remind-text', { count })}</DialogDescription>
        </DialogHeader>
        <p role="alert">{failed ? translate('error-unexpected') : ''}</p>
        <DialogFooter>
          <DialogClose asChild>
            <Button variant="outline">{translate('overview-remind-back')}</Button>
          </DialogClose>
          <Button disabled={pending} onClick={() => void send()}>
            {translate('overview-remind-send')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
