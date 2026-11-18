import { useEffect, useState } from 'react'

/** The secret of a link in the fragment, which the browser never sends. It follows the fragment when someone opens another link in the same tab. */
export function useFragmentSecret(): [string, (secret: string) => void] {
  const [secret, setSecret] = useState(() => window.location.hash.slice(1))
  useEffect(() => {
    const follow = () => setSecret(window.location.hash.slice(1))
    window.addEventListener('hashchange', follow)
    return () => window.removeEventListener('hashchange', follow)
  }, [])
  return [secret, setSecret]
}
