import assert from 'node:assert/strict'
import { mkdir } from 'node:fs/promises'
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright')
const browser = await chromium.launch({ headless: true, args: ['--no-sandbox'] })
const page = await browser.newPage({viewport: {width: 1200, height: 820}, locale: 'zh-CN'})
const artifacts = process.env.ARTIFACT_DIR || '/tmp/banewfn-workbench-ui'
await mkdir(artifacts, {recursive: true})
const errors = []
page.on('pageerror', error => errors.push(error.message))
let sent
let inspections = 0
const status = {format: 'multiwfn-workflows', version: 1, available: true, session: null, runner: 'runner', library: '\\\\?\\C:\\BaneWfn\\scripts', results: '\\\\?\\D:\\分析\\苯_workflows', wavefunction: '\\\\?\\D:\\分析\\苯.fch', multiwfn: 'engine', scripts: [{id: 'density.bw', name: 'density.bw'}], catalogError: null, job: null}
await page.route('**/api/workflows/**', async route => {
  const request = route.request(), url = new URL(request.url())
  assert.equal(url.searchParams.get('cap'), 'test-capability')
  const action = url.pathname.split('/').at(-1)
  let value = status
  if (action === 'preview') value = {id: 'density.bw', source: '[grid]\n%process\nelectron\nspin\nend\n'}
  if (action === 'inspect') {
    inspections++
    const params = request.postDataJSON().parameters
    value = {version: 1, runnable: true, fields: [0, 1].map(i => ({id: `0/${i}/grid`, name: 'grid', label: 'Grid', labels: {zh: i ? '自旋网格' : '网格精度', en: i ? 'Spin grid' : 'Grid quality'}, group: i ? '1 · grid / spin [1]' : '1 · grid / electron [0]', type: 'select', value: params[`0/${i}/grid`] ?? (i ? '2' : '3'), source: 'placeholder', helps: {zh: '控制生成网格的精细程度', en: 'Controls grid resolution'}, options: ['1', '2', '3'].map((value, index) => ({value, label: value, labels: {zh: ['低精度', '中等精度', '高精度'][index], en: ['Low', 'Medium', 'High'][index]}}))})), tasks: [{task: 0, module: 'grid', commands: `5\n${params['0/0/grid'] ?? '3'}\n2\nq\n`}], diagnostics: []}
  }
  if (action === 'run') {
    sent = request.postDataJSON()
    status.job = {state: 'completed', directory: 'D:\\分析\\苯_workflows\\run', exitCode: 0, message: '', log: 'Analysis completed', files: ['density.cub']}
  }
  await route.fulfill({contentType: 'application/json', body: JSON.stringify(value)})
})
try {
  await page.goto(`${process.env.PREVIEW_URL || 'http://127.0.0.1:5205/'}?workbench=1&cap=test-capability`)
  const panel = page.getByRole('dialog', {name: 'BaneWfn', exact: true})
  await panel.getByLabel('网格精度', {exact:true}).waitFor()
  assert.equal(await panel.getByRole('tab').count(), 3)
  assert.equal(await page.locator('canvas').count(), 0)
  assert.equal(await panel.getByLabel('网格精度', {exact:true}).inputValue(), '3')
  assert.equal(await panel.getByLabel('自旋网格', {exact:true}).inputValue(), '2')
  assert.ok(!(await panel.innerText()).includes('\\\\?\\'))
  const left = await panel.locator('.workflow-library').boundingBox(), full = await panel.locator('.workflow-layout').boundingBox()
  assert.ok(Math.abs(left.width / full.width - .3) < .01)
  await panel.getByLabel('网格精度', {exact:true}).selectOption('1')
  await page.waitForFunction(() => [...document.querySelectorAll('button')].some(b => b.textContent.includes('运行工作流') && !b.disabled))
  assert.equal(await panel.getByLabel('自旋网格', {exact:true}).inputValue(), '2')
  await page.screenshot({path: `${artifacts}/banewfn-parameters.png`})
  await panel.getByRole('button', {name: 'English', exact:true}).click()
  assert.equal(await panel.getByLabel('Grid quality', {exact:true}).inputValue(), '1')
  assert.equal(await panel.getByLabel('Spin grid', {exact:true}).inputValue(), '2')
  assert.equal(await panel.getByLabel('Grid quality', {exact:true}).locator('option:checked').innerText(), 'Low')
  assert.equal(await panel.locator('label[for="parameter-0/0/grid"]').getAttribute('title'), 'Controls grid resolution')
  await page.screenshot({path: `${artifacts}/banewfn-parameters-en.png`})
  await panel.getByRole('button', {name: '中文', exact:true}).click()
  assert.equal(await panel.getByLabel('网格精度', {exact:true}).inputValue(), '1')
  assert.equal(await panel.getByLabel('网格精度', {exact:true}).locator('option:checked').innerText(), '低精度')
  await panel.getByRole('tab', {name: '脚本预览', exact:true}).click()
  await panel.getByLabel('预览内容').selectOption({label:'Multiwfn 输入'})
  assert.ok((await panel.locator('.workflow-source').innerText()).includes('5\n1\n2\nq'))
  await panel.getByRole('button', {name: '运行工作流', exact:true}).click()
  await panel.getByText('Analysis completed', {exact:true}).waitFor()
  assert.deepEqual(sent.parameters, {'0/0/grid': '1'})
  assert.ok(inspections >= 2)
  assert.deepEqual(errors, [])
  await page.screenshot({path: `${artifacts}/banewfn-results.png`})
  console.log('PASS: independent window, bilingual labels/help/options, preserved selections, 30/70 layout, native preview and explicit overrides')
} finally { await browser.close() }
