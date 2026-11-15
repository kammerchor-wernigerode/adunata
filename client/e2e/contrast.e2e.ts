/// <reference lib="dom" />
import { expect, test, type Page } from '@playwright/test'

test.use({ locale: 'en-US' })

/** Contrast ratio of two CSS colors as WCAG defines it. The browser resolves each color through a canvas, so any color syntax works. */
async function contrast(page: Page, foreground: string, background: string): Promise<number> {
  return page.evaluate(
    ([first, second]) => {
      const context = document.createElement('canvas').getContext('2d', { willReadFrequently: true })!
      const luminance = (color: string) => {
        context.clearRect(0, 0, 1, 1)
        context.fillStyle = color
        context.fillRect(0, 0, 1, 1)
        const [red, green, blue] = Array.from(context.getImageData(0, 0, 1, 1).data.slice(0, 3)).map((value: number) => {
          const channel = value / 255
          return channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4
        })
        return 0.2126 * red + 0.7152 * green + 0.0722 * blue
      }
      const [lighter, darker] = [luminance(first), luminance(second)].sort((a, b) => b - a)
      return (lighter + 0.05) / (darker + 0.05)
    },
    [foreground, background],
  )
}

/** The opaque color among the shadows that draw a focus ring. Tailwind puts a transparent offset shadow first. */
function ringColor(boxShadow: string): string {
  const colors = boxShadow.match(/(rgba?|oklch|oklab|lab|color)\([^)]*\)/g) ?? []
  const opaque = colors.filter((color) => !/[,/]\s*0\)$/.test(color))
  if (opaque.length !== 1) throw new Error(`expected one opaque ring in ${boxShadow}`)
  return opaque[0]
}

for (const colorScheme of ['light', 'dark'] as const) {
  test(`controls stand out from the page in ${colorScheme} mode`, async ({ page }) => {
    await page.emulateMedia({ colorScheme })
    await page.goto('/')
    const input = page.locator('input[type="text"], input:not([type])').first()
    await expect(input).toBeVisible()
    const background = await page.evaluate(() => getComputedStyle(document.body).backgroundColor)

    const border = await input.evaluate((element) => getComputedStyle(element).borderTopColor)
    expect(await contrast(page, border, background)).toBeGreaterThanOrEqual(3)

    await input.focus()
    await page.keyboard.press('Shift+Tab')
    await page.keyboard.press('Tab')
    const focused = await input.evaluate((element) => ({
      border: getComputedStyle(element).borderTopColor,
      ring: getComputedStyle(element).boxShadow,
    }))
    expect(await contrast(page, focused.border, background)).toBeGreaterThanOrEqual(3)
    expect(await contrast(page, ringColor(focused.ring), background)).toBeGreaterThanOrEqual(3)
  })
}
