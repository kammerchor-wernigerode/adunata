import type { FieldError as ServerFieldError } from '@/api/generated/model'
import type { Translate } from '@/l10n/translate'

/** The text for a field error that the server reports about a member. */
export function memberErrorMessage(translate: Translate, error: ServerFieldError): string {
  switch (error.code) {
    case 'required':
      return translate('error-field-required')
    case 'tooLong':
      return translate('error-field-too-long')
    case 'taken':
      return translate('error-mail-taken', { conflict: error.conflict ?? '' })
    case 'lastOrganizer':
      return translate('error-last-organizer')
    case 'invalid':
      if (error.field === 'mail') return translate('error-mail-invalid')
      if (error.field === 'organizer') return translate('error-organizer-inactive')
      return translate('error-unexpected')
    default:
      return translate('error-unexpected')
  }
}
