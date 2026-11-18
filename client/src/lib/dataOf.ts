/** The status of a cached answer of the API, or `undefined` for data that is no such answer. */
export function statusOf(data: unknown): number | undefined {
  if (typeof data !== 'object' || data === null || !('status' in data) || typeof data.status !== 'number') return undefined
  return data.status
}

/** The data of a response with status 200, or `undefined` while it loads or while a 401 sends the organizer to the sign-in page. Any other status fails the page, so the route shows its error notice with a retry. */
export function dataOf<R extends { status: number; data: unknown }>(response: R | undefined): Extract<R, { status: 200 }>['data'] | undefined {
  if (response === undefined || response.status === 401) return undefined
  if (response.status !== 200) throw new Error(`The API answered with status ${response.status}`)
  return response.data
}

/** Whether a query failed, either because its request threw or because the API answered with an error status. */
export function hasFailed(state: { status: string; data: unknown }): boolean {
  return state.status === 'error' || (statusOf(state.data) ?? 0) >= 400
}
