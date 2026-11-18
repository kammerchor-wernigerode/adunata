/** The result of an action, or the fallback when it throws. The try block lives here, because React Compiler skips components and hooks that contain one with a finally clause. */
export async function orFallback<T>(action: () => Promise<T>, fallback: T): Promise<T> {
  try {
    return await action()
  } catch {
    return fallback
  }
}

/** Whether an action reported success. A thrown error counts as failure. */
export function succeeded(action: () => Promise<boolean>): Promise<boolean> {
  return orFallback(action, false)
}

/** Whether an action finished without throwing. */
export function finished(action: () => Promise<unknown>): Promise<boolean> {
  return succeeded(async () => {
    await action()
    return true
  })
}
