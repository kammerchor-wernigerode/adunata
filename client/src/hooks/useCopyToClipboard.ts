import { useEffect, useRef, useState } from 'react'
import { finished } from '@/lib/succeeded'

const confirmation = 2000

/** Copies text to the clipboard and remembers for two seconds which key it copied last. A browser without a clipboard counts as a failed copy. */
export function useCopyToClipboard() {
  const [copiedKey, setCopiedKey] = useState<number>()
  const [failed, setFailed] = useState(false)
  const timer = useRef<number>(undefined)
  useEffect(() => () => window.clearTimeout(timer.current), [])

  async function copy(key: number, text: string) {
    window.clearTimeout(timer.current)
    setCopiedKey(undefined)
    setFailed(false)
    if (await finished(() => navigator.clipboard.writeText(text))) {
      setCopiedKey(key)
      window.clearTimeout(timer.current)
      timer.current = window.setTimeout(() => setCopiedKey(undefined), confirmation)
    } else {
      setFailed(true)
    }
  }

  return { copiedKey, failed, copy }
}
