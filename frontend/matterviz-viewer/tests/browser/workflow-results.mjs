// Run against the Vite session preview. Synthetic fields exercise rendering,
// not electronic-structure calculations; no user result files are required.
import assert from 'node:assert/strict'
import { mkdir, writeFile } from 'node:fs/promises'
import path from 'node:path'
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright')
const base = process.env.PREVIEW_URL || 'http://127.0.0.1:5297/'
const artifacts = process.env.ARTIFACT_DIR || '/tmp/multiwfn-workflow-results'
await mkdir(artifacts, { recursive: true })
const cube = (offset) => {
  const size = 72, step = 0.12
  const lines = ['Synthetic rendering regression', 'Two independent positive scalar fields',
    '2 -4.26 -4.26 -4.26', `${size} ${step} 0 0`, `${size} 0 ${step} 0`, `${size} 0 0 ${step}`, '1 0 -0.6 0 0', '1 0 0.6 0 0']
  for (let x = 0; x < size; x++) for (let y = 0; y < size; y++) {
    const row = []
    for (let z = 0; z < size; z++) row.push((0.2 * Math.exp(-((x * step - 4.26 - offset) ** 2 + (y * step - 4.26) ** 2 + (z * step - 4.26) ** 2))).toExponential(6))
    lines.push(row.join(' '))
  }
  return lines.join('\n') + '\n'
}
const fields = { 'regression_ele.cub': cube(-0.5), 'regression_hole.cub': cube(0.5) }
const browser = await chromium.launch({ headless: true, args: ['--no-sandbox', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] })
const page = await browser.newPage({ viewport: { width: 1400, height: 900 } })
const errors = []
page.on('pageerror', error => errors.push(String(error)))
await page.addInitScript(() => {
  localStorage.setItem('multiwfn-ui-language', 'en')
  window.renderLongTasks = []
  new PerformanceObserver(list => window.renderLongTasks.push(...list.getEntries().map(entry => entry.duration))).observe({ type: 'longtask' })
})
await page.route('**/session/render-regression.json', route => route.fulfill({ json: {
  cubes: Object.keys(fields).map(name => ({ path: name, name, isovalue: 0.001 })),
} }))
for (const [name, body] of Object.entries(fields)) await page.route(`**/session/${name}`, route => route.fulfill({ body, contentType: 'text/plain' }))
try {
  const url = new URL(base)
  url.searchParams.set('manifest', '/session/render-regression.json')
  await page.goto(url.href, { waitUntil: 'networkidle', timeout: 120000 })
  await page.locator('.rep-row').first().waitFor()
  await page.evaluate(async () => {
    const moduleUrl = performance.getEntriesByType('resource').map(entry => entry.name).find(url => /\/matterviz\.js/.test(url))
    const mv = await import(moduleUrl)
    window.renderScene = mv.scene_registry.get(document.querySelector('canvas')).scene
    window.surfaceGeometry = id => {
      const group = window.renderScene.getObjectByName(`rep-copies-${id}`)
      const geometries = []
      group?.traverseVisible(mesh => {
        if (mesh.isMesh && mesh.geometry.attributes.position?.count > 0) geometries.push(mesh.geometry.uuid)
      })
      return [...new Set(geometries)]
    }
  })
  await page.waitForFunction(() => window.surfaceGeometry('volume-1').length && window.surfaceGeometry('volume-2').length, {}, { timeout: 15000 })
  const initial = await page.evaluate(() => window.surfaceGeometry('volume-1'))
  await page.screenshot({ path: path.join(artifacts, 'two-surfaces.png') })
  await page.locator('.rep-select').nth(1).click()
  await page.getByRole('spinbutton', { name: 'Isovalue', exact: true }).fill('0.002')
  await page.waitForFunction(old => window.surfaceGeometry('volume-1').length && !window.surfaceGeometry('volume-1').some(id => old.includes(id)), initial)
  await page.getByRole('button', { name: 'Hide regression_ele.cub', exact: true }).click()
  await page.waitForFunction(() => !window.surfaceGeometry('volume-1').length && window.surfaceGeometry('volume-2').length)
  await page.getByRole('button', { name: 'Show regression_ele.cub', exact: true }).click()
  await page.waitForFunction(() => window.surfaceGeometry('volume-1').length)
  const longestTask = await page.evaluate(() => Math.max(0, ...window.renderLongTasks))
  assert.ok(longestTask < Number(process.env.RENDER_TASK_LIMIT_MS || 2000), `UI thread blocked for ${longestTask.toFixed(0)} ms`)
  assert.deepEqual(errors, [])
  await writeFile(path.join(artifacts, 'checks.json'), JSON.stringify({ longestTask, errors, surfaces: 2, isovalueUpdated: true, visibilityUpdated: true }, null, 2))
  console.log(`PASS: two real surface meshes, changed geometry, hide/show; longest UI task ${longestTask.toFixed(0)} ms`)
} finally { await browser.close() }
