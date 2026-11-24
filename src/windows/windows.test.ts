// Testes das telas (montadas com portas FAKE) e dos arquivos de entrada.
import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { FakeBackend, FakeMedia, FakeUpdates } from '../ports/fake'
import { setPorts } from '../ports'
import RegionSelector from './RegionSelector.vue'
import ToolbarApp from './ToolbarApp.vue'
import WebcamApp from './WebcamApp.vue'

let b: FakeBackend
let m: FakeMedia

beforeEach(() => {
  b = new FakeBackend()
  m = new FakeMedia()
  b.tg = {
    monitors: [{ id: 1, name: 'Tela 1', w: 1920, h: 1080, x: 0, y: 0, primary: true }],
    wins: [],
  }
  b.dv = { cams: [{ id: 'c1', name: 'Cam' }], mics: [{ id: 'm1', name: 'Mic' }] }
  setPorts({ backend: b, media: m, updates: new FakeUpdates() })
})

describe('ToolbarApp', () => {
  it('fluxo completo: configurar, gravar, pausar, parar', async () => {
    const w = mount(ToolbarApp)
    await flushPromises()
    expect(w.find('.tgt').exists()).toBe(false) // um monitor: sem linha de alvo
    // painel de configurações
    await w.find('button[title="Configurações"]').trigger('click')
    await flushPromises()
    expect(b.calls.find((c) => c[0] === 'fitBar')![1]).toEqual({ w: 720, h: 184 })
    expect(w.find('.tgt').exists()).toBe(true) // seletor de alvo na linha extra
    await w.find('button[title="Áudio do sistema"]').trigger('click')
    const logs = w.find('button[title="Abrir pasta de logs"]')
    expect(logs.text()).toBe('Abrir logs')
    await logs.trigger('click')
    expect(b.calls.some((c) => c[0] === 'openLogs')).toBe(true)
    await w.find('select[aria-label="Quadros por segundo"]').setValue('60')
    // alvo, câmera e mic
    await w.find('select[aria-label="Alvo"]').setValue('1')
    await w.find('select[aria-label="Sem câmera"]').setValue('c1')
    await w.find('select[aria-label="Sem microfone"]').setValue('m1')
    b.emit('mic', 0.9)
    // área não tem seletor de alvo
    await w.findAll('.seg button')[2].trigger('click')
    expect(w.find('select[aria-label="Alvo"]').exists()).toBe(false)
    await w.findAll('.seg button')[0].trigger('click')
    // grava
    await w.find('button[title="Gravar"]').trigger('click')
    await flushPromises()
    expect(w.find('.clock').text()).toBe('00:00')
    expect(b.calls.filter((c) => c[0] === 'fitBar').at(-1)![1]).toEqual({ w: 150, h: 48 }) // pílula
    expect(b.calls.find((c) => c[0] === 'start')![1]).toMatchObject({ sys: false, fps: 60, micId: 'm1', targetId: 1 })
    // pílula: sem botão de pausa; tudo arrastável menos o botão de parar
    expect(w.find('button[title="Pausar"]').exists()).toBe(false)
    expect(w.find('.bar.pill').attributes('data-tauri-drag-region')).toBeDefined()
    expect(w.find('.dot').attributes('data-tauri-drag-region')).toBeDefined()
    expect(w.find('.clock').attributes('data-tauri-drag-region')).toBeDefined()
    expect(w.find('button[title="Parar"]').attributes('data-tauri-drag-region')).toBeUndefined()
    // para
    await w.find('button[title="Parar"]').trigger('click')
    await flushPromises()
    expect(w.find('.ok').exists()).toBe(true)
    // minimizar e fechar
    await w.find('button[title="Minimizar"]').trigger('click')
    await w.find('button[title="Fechar"]').trigger('click')
    expect(b.calls.map((c) => c[0])).toEqual(expect.arrayContaining(['minimize', 'close']))
    expect(w.find('[data-tauri-drag-region]').exists()).toBe(true)
    // erro
    b.fail = 'Falhou'
    await w.find('button[title="Gravar"]').trigger('click')
    await flushPromises()
    expect(w.find('.err').text()).toBe('Falhou')
    expect(w.find('.err').attributes('data-tauri-drag-region')).toBeDefined()
    w.unmount()
    expect(b.subs('hotkey')).toBe(0)
  })
})

describe('WebcamApp', () => {
  it('mostra vídeo, erro e libera a câmera', async () => {
    const w = mount(WebcamApp)
    await flushPromises()
    expect(w.find('video').exists()).toBe(true)
    expect(w.find('.err').exists()).toBe(false)
    expect(w.find('[data-tauri-drag-region]').exists()).toBe(true)
    w.unmount()
    expect(m.stopped).toBe(1)
    m.fail = true
    const e = mount(WebcamApp)
    await flushPromises()
    expect(e.find('.err').text()).toContain('câmera')
  })
})

describe('RegionSelector', () => {
  it('arrasta, confirma e exibe etiqueta', async () => {
    const w = mount(RegionSelector, { attachTo: document.body })
    expect(w.find('.hint').exists()).toBe(true)
    await w.find('.ov').trigger('mousedown', { clientX: 10, clientY: 20 })
    await w.find('.ov').trigger('mousemove', { clientX: 210, clientY: 120 })
    expect(w.find('.tag').text()).toBe('200 × 100')
    await w.find('.ov').trigger('mouseup')
    // alça sudeste ativa durante o redimensionamento; cursor correspondente
    await w.find('.ov').trigger('mousemove', { clientX: 210, clientY: 120 })
    expect((w.find('.ov').element as HTMLElement).style.cursor).toBe('nwse-resize')
    await w.find('.ov').trigger('mousedown', { clientX: 210, clientY: 120 })
    await w.find('.ov').trigger('mousemove', { clientX: 200, clientY: 110 })
    expect(w.find('.hh.se.act').exists()).toBe(true)
    expect(w.find('.tag').text()).toBe('190 × 90')
    await w.find('.ov').trigger('mouseup')
    await w.find('.ov').trigger('mousemove', { clientX: 100, clientY: 70 })
    expect((w.find('.ov').element as HTMLElement).style.cursor).toBe('move')
    await w.find('.ov').trigger('mousemove', { clientX: 900, clientY: 900 })
    expect((w.find('.ov').element as HTMLElement).style.cursor).toBe('crosshair')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }))
    await flushPromises()
    expect(b.calls[0]).toEqual(['submitRegion', { x: 10, y: 20, w: 190, h: 90 }])
    b.fail = 'x'
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await flushPromises()
    expect(w.find('.err').text()).toBe('x')
    w.unmount()
  })
})

describe('ToolbarApp: atualização', () => {
  const rel = 'https://github.com/klawdyo/kapture/releases/tag/v0.2.0'
  const dot = 'button[aria-label="Atualização disponível"]'
  const status = '[role="status"]'
  const pill = (w: ReturnType<typeof mount>) => w.find(status).text()

  it('ciclo automático: verifica, baixa, pede reinício e reinicia ao clicar', async () => {
    const u = new FakeUpdates()
    setPorts({ backend: b, media: m, updates: u })
    const w = mount(ToolbarApp)
    await flushPromises()
    expect(b.calls.some((c) => c[0] === 'updateStart')).toBe(true)
    expect(pill(w)).toBe('') // nada até o backend responder
    b.emit('upd', { kind: 'checking' })
    await flushPromises()
    expect(pill(w)).toBe('Verificando atualizações…')
    b.emit('upd', { kind: 'downloading', pct: 42 })
    await flushPromises()
    expect(pill(w)).toBe('Atualizando… 42%')
    b.emit('upd', { kind: 'ready', version: '0.2.0' })
    await flushPromises()
    expect(pill(w)).toBe('Reinicie para atualizar')
    await w.find(`${status} button`).trigger('click')
    expect(b.calls.at(-1)).toEqual(['updateApply', undefined])
    expect(u.calls).toEqual([])
    // modo compacto de gravação: sem status
    await w.find('button[title="Gravar"]').trigger('click')
    await flushPromises()
    expect(w.find('.pill').exists()).toBe(true)
    expect(w.find(status).exists()).toBe(false)
    w.unmount()
    expect(b.subs('upd')).toBe(0)
  })
  it('sem atualização ou erro: o texto some sem alarde', async () => {
    const u = new FakeUpdates()
    setPorts({ backend: b, media: m, updates: u })
    const w = mount(ToolbarApp)
    await flushPromises()
    b.emit('upd', { kind: 'checking' })
    b.emit('upd', { kind: 'none' })
    await flushPromises()
    expect(pill(w)).toBe('')
    expect(w.find('.err').exists()).toBe(false)
    w.unmount()
  })
  it('fallback: texto, bolinha, mensagem, Depois e Baixar', async () => {
    const u = new FakeUpdates()
    u.res = { version: '0.2.0', url: rel }
    setPorts({ backend: b, media: m, updates: u })
    const w = mount(ToolbarApp)
    await flushPromises()
    b.emit('upd', { kind: 'fallback' })
    await flushPromises()
    expect(u.calls).toEqual(['0.1.0'])
    expect(pill(w)).toBe('Atualização disponível')
    expect(w.find('[role="dialog"]').exists()).toBe(false)
    await w.find(dot).trigger('click')
    expect(w.find('[role="dialog"]').text()).toContain('Nova: v0.2.0')
    await w.findAll('.act button')[1].trigger('click') // Depois
    expect(w.find('[role="dialog"]').exists()).toBe(false)
    expect(w.find(dot).exists()).toBe(true)
    await w.find(dot).trigger('click')
    await w.findAll('.act button')[0].trigger('click') // Baixar
    expect(b.calls).toContainEqual(['openUrl', rel])
    await w.find('button[title="Gravar"]').trigger('click')
    await flushPromises()
    expect(w.find(dot).exists()).toBe(false)
    w.unmount()
  })
  it('fallback com erro de rede não mostra nada', async () => {
    const u = new FakeUpdates()
    u.fail = new Error('offline')
    setPorts({ backend: b, media: m, updates: u })
    const w = mount(ToolbarApp)
    await flushPromises()
    b.emit('upd', { kind: 'fallback' })
    await flushPromises()
    expect(w.find(dot).exists()).toBe(false)
    expect(pill(w)).toBe('')
    w.unmount()
  })
})

describe('entradas', () => {
  it.each(['../main', '../webcam', '../region'])('%s monta o app', async (f) => {
    vi.resetModules()
    document.body.innerHTML = '<div id="app"></div>'
    // Após resetModules o módulo de portas é novo: reinjeta os FAKEs nele.
    ;(await import('../ports')).setPorts({ backend: b, media: m, updates: new FakeUpdates() })
    await import(/* @vite-ignore */ f)
    await flushPromises()
    expect(document.querySelector('#app')!.children.length).toBeGreaterThan(0)
  })
})
