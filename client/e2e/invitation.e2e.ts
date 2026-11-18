/// <reference lib="dom" />
import AxeBuilder from '@axe-core/playwright'
import { expect, test, type Page } from '@playwright/test'

test.use({ locale: 'en-US' })

function mailpit(): string {
  const url = process.env.ADUNATA_E2E_MAILPIT_URL
  if (!url) throw new Error('ADUNATA_E2E_MAILPIT_URL is missing, run the tests with npm run e2e')
  return url
}

/** The path of the newest link of a kind in Mailpit, such as /invitation#3f9a…, waiting until its mail arrives. */
async function newestLink(kind: string): Promise<string> {
  for (let attempt = 0; attempt < 100; attempt++) {
    const list = (await (await fetch(`${mailpit()}/api/v1/messages?limit=1`)).json()) as { messages: { ID: string }[] }
    if (list.messages.length > 0) {
      const message = (await (await fetch(`${mailpit()}/api/v1/message/${list.messages[0].ID}`)).json()) as { Text: string }
      const found = message.Text.match(new RegExp(`/${kind}#[0-9a-f]{64}`))
      if (found) return found[0]
    }
    await new Promise((resolve) => setTimeout(resolve, 200))
  }
  throw new Error(`no ${kind} link arrived`)
}

function isoDay(offset: number): string {
  const day = new Date()
  day.setDate(day.getDate() + offset)
  return `${day.getFullYear()}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getDate()).padStart(2, '0')}`
}

/** Waits until no finite CSS transition or animation runs any more, so that axe sees the final colors after a change of the color scheme. A transition that a newer one replaces rejects its promise, so the check repeats until none is left. Finished and paused animations do not count. */
async function settle(page: Page): Promise<void> {
  await page.evaluate(async () => {
    const running = () =>
      document
        .getAnimations()
        .filter(
          (animation) =>
            animation.playState === 'running' && animation.effect?.getComputedTiming().endTime !== Infinity,
        )
    while (running().length > 0) {
      await Promise.allSettled(running().map((animation) => animation.finished))
    }
  })
}

async function violations(page: Page): Promise<string[]> {
  await settle(page)
  const results = await new AxeBuilder({ page }).analyze()
  return results.violations.map((violation) => `${violation.id}: ${violation.help}`)
}

/** Whether the page fits a phone 390 px wide with the text at 200 % without scrolling sideways. Measures only once the fonts of the client have loaded, so that no font of the system decides the widths of text. */
async function reflowsAt200Percent(page: Page): Promise<boolean> {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.evaluate(() => {
    document.documentElement.style.fontSize = '200%'
  })
  await page.evaluate(() => document.fonts.ready)
  const fits = await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)
  await page.evaluate(() => {
    document.documentElement.style.fontSize = ''
  })
  await page.setViewportSize({ width: 1280, height: 720 })
  return fits
}

test('organizer invites and a member answers through the personal link', async ({ page, context }) => {
  const policyViolations: string[] = []
  page.on('console', (message) => {
    if (message.text().includes('Content Security Policy')) policyViolations.push(message.text())
  })

  await page.goto('/sign-in')
  await page.getByLabel('Mail address').fill('organizer@example.org')
  await page.getByRole('button', { name: 'Send sign-in link' }).click()
  await expect(page.getByRole('status')).toContainText('sign-in link is on its way')
  await page.goto(await newestLink('sign-in'))
  await page.getByRole('button', { name: 'Sign in' }).click()
  await expect(page.getByRole('heading', { level: 1, name: 'Gatherings' })).toBeVisible()

  await page.getByRole('link', { name: 'Create gathering' }).click()
  await page.getByLabel('Title').fill('Rehearsal weekend')
  await page.getByLabel('Location').fill('Drübeck Abbey')
  await page.getByLabel('First day').fill(isoDay(30))
  await page.getByLabel('Last day').fill(isoDay(32))
  await page.getByRole('button', { name: 'Save' }).click()
  await expect(page.getByRole('heading', { level: 1, name: 'Rehearsal weekend' })).toBeVisible()
  await page.getByRole('button', { name: 'Invite the one active member not invited yet' }).click()
  await expect(page.getByText('One invitation sent.')).toBeVisible()

  const member = await context.newPage()
  await member.goto(await newestLink('invitation'))
  await expect(member.getByRole('heading', { level: 1, name: 'Rehearsal weekend' })).toBeVisible()
  await member.getByRole('button', { name: /Day of arrival/ }).click()
  await member.locator('[role="grid"][aria-label="Day of arrival"] button:not([disabled])').first().click()
  await member.getByLabel('Time of arrival').fill('18:00')
  await member.getByRole('radio', { name: 'I offer beds' }).check()
  await member.getByLabel('How many?').fill('2')
  await member.getByRole('button', { name: 'Save response' }).click()
  await expect(member.getByRole('status').filter({ hasText: 'Response saved' })).toContainText('18:00')

  for (const colorScheme of ['light', 'dark'] as const) {
    await member.emulateMedia({ colorScheme })
    expect(await violations(member), `response form in ${colorScheme} mode`).toEqual([])
    expect(await reflowsAt200Percent(member), `response form at 200 % in ${colorScheme} mode`).toBe(true)
  }

  await page.reload()
  await expect(page.getByRole('cell', { name: /Ada Lovelace/ })).toBeVisible()
  await expect(page.getByText('offers 2')).toBeVisible()
  for (const colorScheme of ['light', 'dark'] as const) {
    await page.emulateMedia({ colorScheme })
    expect(await violations(page), `overview in ${colorScheme} mode`).toEqual([])
    expect(await reflowsAt200Percent(page), `overview at 200 % in ${colorScheme} mode`).toBe(true)
  }
  expect(policyViolations).toEqual([])
})
