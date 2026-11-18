/** Adds the browser APIs that Radix primitives and the scroll restoration of the router call and jsdom lacks. Tests in Node skip it. */
if (typeof window !== 'undefined') {
  if (!('ResizeObserver' in globalThis)) {
    globalThis.ResizeObserver = class {
      observe() {}
      unobserve() {}
      disconnect() {}
    }
  }
  window.scrollTo = () => {}
}
