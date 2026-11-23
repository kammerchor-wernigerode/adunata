import { Download } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useLocale, useTranslate } from '@/l10n/translate'

/** The link that downloads the overview as CSV file, with the notice that the file holds personal data. */
export function DownloadOverview({ gatheringId }: { gatheringId: number }) {
  const translate = useTranslate()
  const { locale } = useLocale()
  return (
    <div className="flex flex-wrap items-center gap-3">
      <Button asChild variant="outline">
        <a href={`/api/gatherings/${gatheringId}/overview.csv?language=${locale}`} aria-describedby="download-overview-notice">
          <Download aria-hidden="true" />
          {translate('overview-download')}
        </a>
      </Button>
      <p id="download-overview-notice" className="text-sm text-muted-foreground">
        {translate('overview-download-notice')}
      </p>
    </div>
  )
}
