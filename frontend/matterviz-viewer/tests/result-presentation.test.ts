import assert from 'node:assert/strict'
import test from 'node:test'
import { compact_result_name, group_result_entries, result_field, result_pairs } from '../src/result-presentation.ts'

test('hole/electron aliases use distinct colors and pair by folder and full prefix', () => {
  const entries = ['a/mol_s1_ele.cub', 'a/mol_s2_ele.cub', 'b/mol_s1_hole.cub', 'a/mol_s1_ole.cub', 'a/mol_s2_hole.cub'].map((name, i) => ({ name, path: `${i}_staged.cub` }))
  assert.deepEqual([...result_pairs(entries)], ['a/mol_s1', 'a/mol_s2'])
  assert.deepEqual(group_result_entries(entries).map(e => e.name), [entries[3].name, entries[0].name, entries[4].name, entries[1].name, entries[2].name])
  for (const suffix of ['ole', 'hole', 'HOLE.cube']) assert.equal(result_field({ path: `mol_${suffix}` })?.color, '#2563eb')
  for (const suffix of ['ele', 'electron', 'ELE.cub']) assert.equal(result_field({ path: `mol_${suffix}` })?.color, '#16a34a')
  assert.equal(result_field({ path: 'mol_den.cub' }), undefined)
  assert.equal(result_field({ path: 'mol_ele.cub', role: 'orbital' }), undefined)
  assert.equal(result_field({ path: 'mol_ele.cub', analysisKind: 'esp-potential' }), undefined)
})

test('incomplete and ambiguous pairs do not group unrelated datasets', () => {
  const entries = ['x_ole.cub', 'x_hole.cub', 'x_ele.cub', 'y_electron.cub'].map(path => ({ path }))
  assert.equal(result_pairs(entries).size, 0)
  assert.deepEqual(group_result_entries(entries), entries)
  assert.equal(result_field({ path: 'different.cub', name: 'folder/真实分子_s1_hole' })?.group, 'folder/真实分子_s1')
})

test('compact display preserves distinguishing suffixes and never changes stored filenames', () => {
  const full = 'very/long/folder/a_very_long_molecule_identifier_with_an_excited_state_s12_hole.cub'
  const display = compact_result_name(full)
  assert.ok(display.endsWith('_s12_hole.cub'))
  assert.ok(display.includes('…'))
  assert.equal(Array.from(display).length, 34)
  assert.equal(compact_result_name('folder/分子_s1_ele.cub'), '分子_s1_ele.cub')
})
