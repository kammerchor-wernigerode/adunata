import { createContext, useContext } from 'react'
import type { MessageArgs } from './bundle'

export type Translate = (id: string, args?: MessageArgs) => string

export const TranslateContext = createContext<Translate>((id) => id)

export function useTranslate(): Translate {
  return useContext(TranslateContext)
}
