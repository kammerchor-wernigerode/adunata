import { useLegalLinks } from '@/api/generated/api'
import { useTranslate } from '@/l10n/translate'

export function LegalFooter() {
  const translate = useTranslate()
  const links = useLegalLinks().data?.data
  const entries = [
    { url: links?.privacyUrl, label: translate('privacy-notice') },
    { url: links?.imprintUrl, label: translate('imprint') },
  ].filter((entry): entry is { url: string; label: string } => Boolean(entry.url))
  if (entries.length === 0) return null
  return (
    <footer className="border-t border-border">
      <ul className="mx-auto flex max-w-5xl flex-wrap gap-6 px-4 py-4 text-sm">
        {entries.map((entry) => (
          <li key={entry.url}>
            <a href={entry.url} className="underline underline-offset-4">
              {entry.label}
            </a>
          </li>
        ))}
      </ul>
    </footer>
  )
}
