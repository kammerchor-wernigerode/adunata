import { configure } from '@testing-library/react'

/**
 * Adds the browser APIs that Radix primitives and the scroll restoration of the router call and jsdom lacks.
 * Gives queries of Testing Library enough time to find their state on a loaded machine. Tests in Node skip it.
 */
if (typeof window !== 'undefined') {
  configure({ asyncUtilTimeout: 10_000 })
  if (!('ResizeObserver' in globalThis)) {
    globalThis.ResizeObserver = class {
      observe() {}
      unobserve() {}
      disconnect() {}
    }
  }
  window.scrollTo = () => {}
}
