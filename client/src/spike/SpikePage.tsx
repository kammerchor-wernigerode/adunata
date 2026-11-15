import { Page } from '@/layout/Page'
import { useTranslate } from '@/l10n/translate'
import { SpikeForm } from './SpikeForm'

export function SpikePage() {
  const translate = useTranslate()
  return (
    <Page title={translate('spike-title')}>
      <SpikeForm />
    </Page>
  )
}
