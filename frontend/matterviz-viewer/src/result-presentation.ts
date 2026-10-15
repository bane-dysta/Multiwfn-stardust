import type { ManifestEntry } from './manifest.ts'

type Entry = Pick<ManifestEntry, 'name' | 'path' | 'role' | 'mode' | 'analysisKind'>
export const RESULT_COLORS = { hole: '#2563eb', electron: '#16a34a' } as const
export const result_filename = (entry: Entry): string => {
  if (entry.name) return entry.name.replaceAll('\\', '/')
  try { return decodeURIComponent(entry.path).replaceAll('\\', '/') } catch { return entry.path }
}
export const result_field = (entry: Entry | undefined) => {
  if (!entry || entry.role === 'orbital' || entry.mode === 'signed' || entry.analysisKind) return undefined
  const file = result_filename(entry)
  const match = /^(.*)_(ole|hole|ele|electron)(?:\.cube?)?$/i.exec(file)
  if (!match) return undefined
  const kind = /^(ole|hole)$/i.test(match[2]) ? 'hole' as const : 'electron' as const
  return { kind, group: match[1], label: kind === 'hole' ? 'Hole density' : 'Electron distribution', color: RESULT_COLORS[kind] }
}

/** Match exact folder + prefix; ambiguous or incomplete pairs stay independent. */
export const result_pairs = (entries: Entry[]): Set<string> => {
  const counts = new Map<string, { hole: number; electron: number }>()
  for (const entry of entries) {
    const field = result_field(entry)
    if (!field) continue
    const count = counts.get(field.group) ?? { hole: 0, electron: 0 }
    count[field.kind]++
    counts.set(field.group, count)
  }
  return new Set([...counts].filter(([, count]) => count.hole === 1 && count.electron === 1).map(([key]) => key))
}

/** Keep each matched pair adjacent, hole first, without altering either field. */
export const group_result_entries = <T extends Entry>(entries: T[]): T[] => {
  const pairs = result_pairs(entries), emitted = new Set<string>()
  return entries.flatMap(entry => {
    const field = result_field(entry)
    if (!field || !pairs.has(field.group)) return [entry]
    if (emitted.has(field.group)) return []
    emitted.add(field.group)
    return ['hole', 'electron'].map(kind => entries.find(candidate => {
      const other = result_field(candidate)
      return other?.group === field.group && other.kind === kind
    })!)
  })
}

/** Drop display-only folders and abbreviate the middle, retaining type/state suffixes. */
export const compact_result_name = (name: string, limit = 34): string => {
  const basename = name.replaceAll('\\', '/').split('/').at(-1) || name
  const chars = Array.from(basename)
  if (chars.length <= limit) return basename
  const tail = Math.ceil((limit - 1) / 2)
  return chars.slice(0, limit - tail - 1).join('') + '…' + chars.slice(-tail).join('')
}
