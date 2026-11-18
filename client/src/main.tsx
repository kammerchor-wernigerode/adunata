import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider } from '@tanstack/react-router'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import { LocalizationProvider } from './l10n/LocalizationProvider'
import { createAppRouter } from './router'

const queryClient = new QueryClient()
const router = createAppRouter(queryClient)

const root = document.getElementById('root')
if (!root) throw new Error('index.html has no element with the id root')

createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <LocalizationProvider>
        <RouterProvider router={router} />
      </LocalizationProvider>
    </QueryClientProvider>
  </StrictMode>,
)
