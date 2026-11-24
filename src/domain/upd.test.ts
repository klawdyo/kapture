// Testes da lógica pura de atualização.
import { describe, expect, it } from 'vitest'
import { bare, cmpVer, okUrl, parseRel, parseVer, pickNew, REL_URL } from './upd'

const url = `${REL_URL}tag/v1.2.3`

describe('parseVer', () => {
  it('ignora o prefixo v e metadados de build', () => {
    expect(parseVer('v1.2.3')).toEqual([1, 2, 3])
    expect(parseVer('V10.0.7')).toEqual([10, 0, 7])
    expect(parseVer(' 0.1.0 ')).toEqual([0, 1, 0])
    expect(parseVer('1.2.3+build.5')).toEqual([1, 2, 3])
  })
  it('rejeita prerelease e formatos inválidos', () => {
    for (const t of ['1.2.3-beta.1', '1.2', 'abc', '', 'v1.2.3.4']) expect(parseVer(t)).toBeNull()
  })
})

describe('cmpVer', () => {
  it('compara por maior, menor e correção', () => {
    expect(cmpVer([1, 0, 0], [0, 9, 9])).toBeGreaterThan(0)
    expect(cmpVer([1, 1, 0], [1, 2, 0])).toBeLessThan(0)
    expect(cmpVer([1, 2, 3], [1, 2, 4])).toBeLessThan(0)
    expect(cmpVer([1, 2, 3], [1, 2, 3])).toBe(0)
    expect(cmpVer([1, 10, 0], [1, 9, 0])).toBeGreaterThan(0) // numérico, não textual
  })
})

describe('okUrl e bare', () => {
  it('aceita só páginas de release do kapture', () => {
    expect(okUrl(url)).toBe(true)
    for (const u of [
      '', 'http://github.com/klawdyo/kapture/releases/tag/v1', 'https://github.com/klawdyo/kapture/issues/1',
      `${REL_URL}../x`, `${REL_URL}a b`, `${REL_URL}a"b`, `${REL_URL}a\\b`, 'https://evil.com/klawdyo/kapture/releases/',
    ]) expect(okUrl(u)).toBe(false)
  })
  it('remove o v', () => {
    expect(bare('v1.2.3')).toBe('1.2.3')
    expect(bare('1.2.3')).toBe('1.2.3')
  })
})

describe('parseRel', () => {
  it('lê tag_name e html_url', () => {
    expect(parseRel({ tag_name: 'v1.2.3', html_url: url })).toEqual({ version: '1.2.3', url })
  })
  it('descarta respostas inválidas', () => {
    for (const d of [
      null, 'x', 42, {}, { tag_name: 'v1.2.3' }, { html_url: url }, { tag_name: 1, html_url: url },
      { tag_name: 'v1.2.3', html_url: 2 },
      { tag_name: 'v1.2.3', html_url: url, draft: true },
      { tag_name: 'v1.2.3', html_url: url, prerelease: true },
      { tag_name: 'v1.2.3-rc.1', html_url: url },
      { tag_name: 'v1.2.3', html_url: 'https://evil.com/' },
    ]) expect(parseRel(d)).toBeNull()
  })
})

describe('pickNew', () => {
  const u = (v: string): { version: string; url: string } => ({ version: v, url: `${REL_URL}tag/v${v}` })
  it('escolhe a mais nova maior que a instalada', () => {
    expect(pickNew('0.1.0', [u('0.2.0'), u('0.10.0'), u('0.9.0')])).toEqual(u('0.10.0'))
  })
  it('null quando não há mais nova, igual, lista vazia ou versão ilegível', () => {
    expect(pickNew('0.2.0', [u('0.2.0'), u('0.1.0')])).toBeNull()
    expect(pickNew('0.2.0', [])).toBeNull()
    expect(pickNew('dev', [u('9.0.0')])).toBeNull()
    expect(pickNew('0.1.0', [{ version: 'x', url: '' }])).toBeNull()
  })
})
