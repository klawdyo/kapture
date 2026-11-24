// Testes das portas: FAKE, REAL (com @tauri-apps/api mockado) e provedor.
import { beforeEach, describe, expect, it, vi } from 'vitest'

const invoke = vi.fn()
const handlers: Record<string, (e: { payload: unknown }) => void> = {}
const listen = vi.fn(async (ev: string, cb: (e: { payload: unknown }) => void) => {
  handlers[ev] = cb
  return () => {}
})
let resized: (e: { payload: { width: number; height: number } }) => void = () => {}
const setSize = vi.fn(async () => {})
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...a: unknown[]) => invoke(...a) }))
vi.mock('@tauri-apps/api/event', () => ({ listen: (...a: [string, never]) => listen(...a) }))
vi.mock('@tauri-apps/api/window', () => ({
  PhysicalSize: class { constructor(public width: number, public height: number) {} },
  getCurrentWindow: () => ({
    setSize,
    onResized: async (cb: typeof resized) => { resized = cb; return () => {} },
  }),
}))

import { FakeBackend, FakeMedia } from './fake'
import { ports, setPorts } from './index'
import { realBackend, realMedia } from './real'

describe('FakeBackend', () => {
  it('simula comandos e estado', async () => {
    const f = new FakeBackend()
    expect(await f.targets()).toEqual({ monitors: [], wins: [] })
    expect(await f.devices()).toEqual({ cams: [], mics: [] })
    await f.toggleCam(true, 'c1')
    expect(await f.pickRegion()).toBeNull()
    await f.submitRegion(null)
    await f.fitSquare(300)
    await f.start({ mode: 'screen', sys: true, fps: 30 })
    expect((await f.state()).state).toBe('rec')
    await f.pause()
    expect(f.st.state).toBe('paused')
    await f.resume()
    expect(f.st.state).toBe('rec')
    expect(await f.stop()).toBe(f.path)
    expect(f.calls.map((c) => c[0])).toContain('fitSquare')
  })
  it('falha quando pedido', async () => {
    const f = new FakeBackend()
    f.fail = 'ruim'
    await expect(f.state()).rejects.toBe('ruim')
  })
  it('emite eventos e cancela assinatura', async () => {
    const f = new FakeBackend()
    const got: unknown[] = []
    const offs = await Promise.all([
      f.onMic((l) => got.push(l)),
      f.onState((s) => got.push(s.state)),
      f.onHotkey(() => got.push('hk')),
      f.onResize((w, h) => got.push([w, h])),
    ])
    f.emit('mic', 0.5)
    f.emit('state', { state: 'rec', secs: 1 })
    f.emit('hotkey')
    f.emit('resize', 1, 2)
    expect(got).toEqual([0.5, 'rec', 'hk', [1, 2]])
    expect(f.subs('mic')).toBe(1)
    offs.forEach((o) => o())
    expect(f.subs('mic')).toBe(0)
  })
})

describe('FakeMedia', () => {
  it('abre e para trilhas; pode falhar', async () => {
    const m = new FakeMedia()
    const s = await m.openCam('x')
    s.getTracks().forEach((t) => t.stop())
    expect(m.calls).toEqual(['x'])
    expect(m.stopped).toBe(1)
    m.fail = true
    await expect(m.openCam()).rejects.toThrow('sem câmera')
  })
})

describe('provedor', () => {
  it('troca as portas', () => {
    const orig = ports()
    expect(orig.backend).toBe(realBackend)
    const p = { backend: new FakeBackend(), media: new FakeMedia() }
    setPorts(p)
    expect(ports()).toBe(p)
    setPorts(orig)
  })
})

describe('realBackend', () => {
  beforeEach(() => { invoke.mockReset(); invoke.mockResolvedValue('ok') })
  it('mapeia comandos para invoke', async () => {
    const r = { x: 1, y: 2, w: 3, h: 4 }
    const cfg = { mode: 'screen', sys: true, fps: 30 } as const
    await realBackend.targets()
    await realBackend.devices()
    await realBackend.toggleCam(true, 'c')
    await realBackend.pickRegion()
    await realBackend.submitRegion(r)
    await realBackend.start(cfg)
    await realBackend.pause()
    await realBackend.resume()
    await realBackend.stop()
    await realBackend.state()
    expect(invoke.mock.calls).toEqual([
      ['list_targets'], ['list_devices'], ['toggle_cam', { visible: true, devId: 'c' }],
      ['pick_region'], ['submit_region', { region: r }], ['start_rec', { cfg }],
      ['pause_rec'], ['resume_rec'], ['stop_rec'], ['get_state'],
    ])
  })
  it('assina eventos entregando o payload', async () => {
    const got: unknown[] = []
    await realBackend.onMic((l) => got.push(l))
    await realBackend.onState((s) => got.push(s))
    await realBackend.onHotkey(() => got.push('hk'))
    handlers['mic-level']({ payload: { level: 0.3 } })
    handlers['rec-state']({ payload: { state: 'rec', secs: 2 } })
    handlers['hotkey']({ payload: null })
    expect(got).toEqual([0.3, { state: 'rec', secs: 2 }, 'hk'])
  })
  it('redimensiona a janela', async () => {
    const got: number[][] = []
    await realBackend.onResize((w, h) => got.push([w, h]))
    resized({ payload: { width: 10, height: 20 } })
    await realBackend.fitSquare(50)
    expect(got).toEqual([[10, 20]])
    expect(setSize).toHaveBeenCalledWith({ width: 50, height: 50 })
  })
})

describe('realMedia', () => {
  it('pede a câmera padrão ou específica', async () => {
    const gum = vi.fn().mockResolvedValue('stream')
    Object.defineProperty(navigator, 'mediaDevices', { value: { getUserMedia: gum }, configurable: true })
    await realMedia.openCam()
    await realMedia.openCam('abc')
    expect(gum).toHaveBeenNthCalledWith(1, { video: true, audio: false })
    expect(gum).toHaveBeenNthCalledWith(2, { video: { deviceId: { exact: 'abc' } }, audio: false })
  })
})
