// Optional real-host acceptance. Launch with the session_live.rs fixture library.
// Verifies its harmless raw input before running: only a global menu parameter.
import assert from 'node:assert/strict'
import { mkdir } from 'node:fs/promises'
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright')
assert.ok(process.env.LIVE_SESSION_URL, 'LIVE_SESSION_URL must point to the fixture desktop')
const artifacts = process.env.ARTIFACT_DIR || '/tmp/multiwfn-session-ui'
await mkdir(artifacts, { recursive: true })
const browser = await chromium.launch({ headless: true, args: ['--no-sandbox'] })
const page = await browser.newPage({ viewport: { width: 1280, height: 900 } })
try {
  const url = new URL(process.env.LIVE_SESSION_URL)
  url.searchParams.set('ownedSession', '1')
  await page.goto(url.href, { waitUntil: 'networkidle' })
  await page.getByRole('button', { name: 'Close session', exact: true }).waitFor()
  await page.getByRole('button', { name: 'Workflows', exact: true }).click()
  const panel = page.getByRole('dialog', { name: 'Workflows', exact: true })
  await panel.getByText(/Current session · PID/).waitFor()
  const identity = await panel.locator('.workflow-session').textContent()
  await panel.locator('button.workflow-script[title="set.bw"]').click()
  await panel.getByRole('tab', { name: 'Script preview', exact: true }).click()
  await page.waitForFunction(() => document.querySelector('.workflow-source')?.textContent?.includes('0.125'))
  assert.equal((await panel.locator('.workflow-source').textContent()).trim(), '%raw\n1000\n5\n0.125\nend')
  await panel.getByRole('button', { name: 'Run workflow', exact: true }).click()
  await panel.getByRole('status').filter({ hasText: 'Completed' }).waitFor({ timeout: 30000 })
  assert.equal(await panel.locator('.workflow-session').textContent(), identity)
  await page.screenshot({ path: `${artifacts}/real-session-workflow.png` })
  console.log(`PASS: real GUI host, current ${identity}, raw block execution, result and refresh`)
} finally {
  await browser.close()
}
