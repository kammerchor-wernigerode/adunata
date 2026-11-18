import { AxeBuilder } from '@axe-core/playwright'
import { expect, test } from '@playwright/test'

test.use({ locale: 'en-US' })

test('switches the language and stays accessible', async ({ page }) => {
  const violations: string[] = []
  page.on('console', (message) => {
    if (message.text().includes('Content Security Policy')) violations.push(message.text())
  })
  await page.goto('/')
  await expect(page.getByLabel('Language')).toBeVisible()
  await expect(page.locator('html')).toHaveAttribute('lang', 'en')

  await page.getByLabel('Language').click()
  await page.getByRole('option', { name: 'Deutsch' }).click()
  await expect(page.getByLabel('Sprache')).toBeVisible()
  await expect(page.locator('html')).toHaveAttribute('lang', 'de')

  await page.reload()
  await expect(page.getByLabel('Sprache')).toBeVisible()

  expect(violations).toEqual([])

  const results = await new AxeBuilder({ page }).analyze()
  expect(results.violations.map((violation) => `${violation.id}: ${violation.help}`)).toEqual([])
})
