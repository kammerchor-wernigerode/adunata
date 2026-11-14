import AxeBuilder from '@axe-core/playwright'
import { expect, test } from '@playwright/test'

test.use({ locale: 'en-US' })

test('switches the language and stays accessible', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByLabel('Language')).toBeVisible()
  await expect(page.locator('html')).toHaveAttribute('lang', 'en')

  await page.getByLabel('Language').selectOption('de')
  await expect(page.getByLabel('Sprache')).toBeVisible()
  await expect(page.locator('html')).toHaveAttribute('lang', 'de')

  await page.reload()
  await expect(page.getByLabel('Sprache')).toBeVisible()

  const results = await new AxeBuilder({ page }).analyze()
  expect(results.violations.map((violation) => `${violation.id}: ${violation.help}`)).toEqual([])
})
