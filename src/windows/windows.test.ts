// Testes das telas (montadas com portas FAKE) e dos arquivos de entrada.
import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { FakeBackend, FakeMedia } from '../ports/fake'
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
  setPorts({ backend: b, media: m })
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
    expect(b.calls.filter((c) => c[0] === 'fitBar').at(-1)![1]).toEqual({ w: 260, h: 56 }) // pílula
    expect(b.calls.find((c) => c[0] === 'start')![1]).toMatchObject({ sys: false, fps: 60, micId: 'm1', targetId: 1 })
    // pausa / retoma
    await w.find('button[title="Pausar"]').trigger('click')
    await flushPromises()
    expect(w.find('.dot.off').exists()).toBe(true)
    await w.find('button[title="Retomar"]').trigger('click')
    await flushPromises()
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
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }))
    await flushPromises()
    expect(b.calls[0]).toEqual(['submitRegion', { x: 10, y: 20, w: 200, h: 100 }])
    b.fail = 'x'
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await flushPromises()
    expect(w.find('.err').text()).toBe('x')
    w.unmount()
  })
})

describe('entradas', () => {
  it.each(['../main', '../webcam', '../region'])('%s monta o app', async (f) => {
    vi.resetModules()
    document.body.innerHTML = '<div id="app"></div>'
    // Após resetModules o módulo de portas é novo: reinjeta os FAKEs nele.
    ;(await import('../ports')).setPorts({ backend: b, media: m })
    await import(/* @vite-ignore */ f)
    await flushPromises()
    expect(document.querySelector('#app')!.children.length).toBeGreaterThan(0)
  })
})
