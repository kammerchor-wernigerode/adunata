import { useQueryClient } from '@tanstack/react-query'
import { useRef, useState } from 'react'
import { getCurrentSessionQueryKey } from '@/api/generated/api'
import { succeeded } from '@/lib/succeeded'

/** Runs one request at a time for a control and remembers whether the last one failed, so the control can disable itself and name the failure. A failure checks the session again, so an ended session leads to the sign-in page. */
export function useSubmission() {
  const queryClient = useQueryClient()
  const running = useRef(false)
  const [pending, setPending] = useState(false)
  const [failed, setFailed] = useState(false)

  async function submit(action: () => Promise<boolean>) {
    if (running.current) return
    running.current = true
    setPending(true)
    setFailed(false)
    const ok = await succeeded(action)
    running.current = false
    setPending(false)
    setFailed(!ok)
    if (!ok) void queryClient.invalidateQueries({ queryKey: getCurrentSessionQueryKey() })
  }

  return { pending, failed, submit, reset: () => setFailed(false) }
}
