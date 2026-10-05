import { test, expect } from '@playwright/test'

// RA-014: at 1280x720 the Q-style grid was clipped (Khối 12 and the last totals column hidden).
const viewports = [
  { width: 1024, height: 700 },
  { width: 1280, height: 720 },
  { width: 1440, height: 900 },
  { width: 1920, height: 1080 },
]

for (const vp of viewports) {
  test(`Q grid and totals panel are not clipped at ${vp.width}x${vp.height}`, async ({
    page,
  }) => {
    await page.setViewportSize(vp)
    await page.goto('/#/assignments')
    await page.getByTestId('q-plan-grid-root').waitFor({ timeout: 15000 })

    const m = await page.evaluate(() => {
      const table = document.querySelector('[data-testid=q-plan-grid-table]') as HTMLElement
      const wrap = table.parentElement as HTMLElement
      const totals = document.querySelector('[data-testid=q-plan-totals-panel]') as HTMLElement
      const tr = table.getBoundingClientRect()
      const wr = wrap.getBoundingClientRect()
      const tp = totals.getBoundingClientRect()
      return {
        tableRight: tr.right,
        wrapRight: wr.right,
        wrapScrollOverflow: wrap.scrollWidth - wrap.clientWidth,
        totalsRight: tp.right,
        totalsScrollOverflow: totals.scrollWidth - totals.clientWidth,
        viewportWidth: window.innerWidth,
        pageOverflow: document.documentElement.scrollWidth - window.innerWidth,
      }
    })

    // every grade column of the grid is inside its container (no sideways scrolling)
    expect(m.wrapScrollOverflow).toBeLessThanOrEqual(1)
    expect(m.tableRight).toBeLessThanOrEqual(m.wrapRight + 1)
    // the totals panel (all columns up to the last exam column) is fully visible
    expect(m.totalsScrollOverflow).toBeLessThanOrEqual(1)
    expect(m.totalsRight).toBeLessThanOrEqual(m.viewportWidth)
    expect(m.pageOverflow).toBeLessThanOrEqual(0)
  })
}
