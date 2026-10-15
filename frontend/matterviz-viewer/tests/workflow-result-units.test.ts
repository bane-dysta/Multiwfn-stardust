import assert from 'node:assert/strict'
import test from 'node:test'
import type { VolumetricData } from 'matterviz'
import { KCAL_PER_HARTREE, normalize_esp_volume, esp_legend_ticks, estimate_esp_range, volume_dimensions, extractEspExtrema } from '../src/esp.ts'

test('kcal Cube display matches atomic-unit ESP without modifying source values', () => {
  const entry = { path: 'benzene_kcal_esp.cub', analysisKind: 'esp-potential', valueUnit: 'kcal/mol' as const }
  for (const grid of [[[[ -10, 20 ]]], { data: new Float64Array([-10, 20]), dimensions: [1, 1, 2] as [number, number, number], order: 'z-fastest' as const }]) {
    const input: VolumetricData = { grid, grid_dims: [1, 1, 2], lattice: [[1, 0, 0], [0, 1, 0], [0, 0, 1]], origin: [0, 0, 0], periodic: false, data_range: { min: -10, max: 20, mean: 5, abs_max: 20 } }
    const original = structuredClone(input)
    const output = normalize_esp_volume(input, entry)
    assert.deepEqual(input, original)
    assert.equal(output.data_range.max, 20 / KCAL_PER_HARTREE)
    const values = Array.isArray(output.grid) ? output.grid[0][0] : [...output.grid.data]
    assert.deepEqual(values, [-10 / KCAL_PER_HARTREE, 20 / KCAL_PER_HARTREE])
    const ticks = esp_legend_ticks(output.data_range.min, output.data_range.max)
    assert.ok(Math.abs(ticks[0].kcalMolPerElectron - 20) < 1e-12)
    assert.ok(Math.abs(ticks.at(-1)!.kcalMolPerElectron + 10) < 1e-12)
    assert.equal(normalize_esp_volume(input, { ...entry, valueUnit: 'au' }), input)
    assert.equal(normalize_esp_volume(input, { ...entry, analysisKind: 'esp-density' }), input)
  }
})

test('parsed Cube flat grids preserve surface range and extrema for both storage orders', () => {
  const density = { grid_dims: [3, 3, 3], origin: [0, 0, 0], lattice: [[2, 0, 0], [0, 2, 0], [0, 0, 2]], periodic: false,
    grid: Array.from({length:3}, (_, x) => Array.from({length:3}, (_, y) => Array.from({length:3}, (_, z) => x === 1 && y === 1 && z === 1 ? 1 : 0))) }
  const potential = { ...density, grid: density.grid.map((p, x) => p.map((r, y) => r.map((_, z) => (x * 2 + y * 3 - z * 4) / 100))) }
  const expected = estimate_esp_range(density, potential, .5)
  assert.ok(expected.sampleCount > 0 && !expected.fallback)
  for (const order of ['x-fastest', 'z-fastest'] as const) {
    const flat = (volume: typeof density) => {
      const data = new Float64Array(27)
      for (let x=0;x<3;x++) for (let y=0;y<3;y++) for (let z=0;z<3;z++) data[order === 'x-fastest' ? x + 3*(y + 3*z) : z + 3*(y + 3*x)] = volume.grid[x][y][z]
      return {...volume, grid: {data, dimensions:[3,3,3], order}}
    }
    assert.deepEqual(estimate_esp_range(flat(density), flat(potential), .5), expected)
    assert.deepEqual(extractEspExtrema(flat(density), flat(potential), .5), extractEspExtrema(density, potential, .5))
    assert.equal(volume_dimensions({...flat(density), grid_dims:[2,3,3]}), undefined)
  }
})
