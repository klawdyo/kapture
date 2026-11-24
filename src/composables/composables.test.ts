// Testes dos composables (portas FAKE).
import { beforeEach, describe, expect, it } from 'vitest'
import { FakeBackend, FakeMedia } from '../ports/fake'
import { setPorts } from '../ports'
import { useRegion } from './useRegion'
import { useToolbar } from './useToolbar'
import { useWebcam } from './useWebcam'

let b: FakeBackend
let m: FakeMedia
const flush = () => new Promise((r) => setTimeout(r))

beforeEach(() => {
  b = new FakeBackend()
  m = new FakeMedia()
  b.tg = {
    monitors: [{ id: 1, name: 'Tela 1', w: 1920, h: 1080, x: 0, y: 0, primary: true }],
    wins: [{ id: 9, title: 'Notas' }],
  }
  b.dv = { cams: [{ id: 'c1', name: 'Cam' }], mics: [{ id: 'm1', name: 'Mic' }] }
  setPorts({ backend: b, media: m })
})

describe('useToolbar', () => {
  it('inicializa dispositivos, alvo e assinaturas', async () => {
    const t = useToolbar()
    await t.init()
    expect(t.devs.value.cams).toHaveLength(1)
    expect(t.tid.value).toBe('1')
    b.emit('mic', 0.8)
    expect(t.lvl.value).toBe(0) // sem microfone escolhido
    t.mic.value = 'm1'
    expect(t.lvl.value).toBe(0.8)
    b.emit('state', { state: 'rec', secs: 75 })
    expect(t.active.value).toBe(true)
    expect(t.clock.value).toBe('01:15')
    t.dispose()
    expect(b.subs('mic')).toBe(0)
    expect(b.calls.at(-1)).toEqual(['micStop', undefined]) // descartar desliga o medidor
  })
  it('medidor liga ao escolher o microfone e desliga com nenhum', async () => {
    const t = useToolbar()
    await t.setMic('m1')
    await t.setMic('m2')
    await t.setMic('')
    expect(t.mic.value).toBe('')
    expect(b.calls.filter((c) => c[0].startsWith('mic')).map((c) => [c[0], c[1]])).toEqual([
      ['micStart', 'm1'], ['micStart', 'm2'], ['micStop', undefined],
    ])
    b.fail = 'sem mic'
    await t.setMic('m1')
    expect(t.err.value).toBe('sem mic')
    t.dispose() // falha ao desligar é ignorada
    await flush()
  })
  it('abre a pasta de logs e mostra a falha', async () => {
    const t = useToolbar()
    await t.openLogs()
    expect(b.calls.at(-1)![0]).toBe('openLogs')
    b.fail = 'Não foi possível abrir a pasta de logs'
    await t.openLogs()
    expect(t.err.value).toBe('Não foi possível abrir a pasta de logs')
  })
  it('mostra erro de init', async () => {
    b.fail = 'Falha ao listar'
    const t = useToolbar()
    await t.init()
    expect(t.err.value).toBe('Falha ao listar')
  })
  it('troca de modo escolhe alvo', () => {
    const t = useToolbar()
    t.setMode('area')
    expect(t.tid.value).toBe('')
    t.setMode('screen')
    expect(t.tid.value).toBe('')
  })
  it('grava, pausa, retoma e para', async () => {
    const t = useToolbar()
    await t.init()
    t.mic.value = 'm1'
    await t.toggle() // inicia
    expect(b.calls.find((c) => c[0] === 'start')![1]).toEqual({
      mode: 'screen', sys: true, fps: 30, targetId: 1, micId: 'm1',
    })
    expect(t.rec.value.state).toBe('rec')
    await t.pause()
    expect(t.paused.value).toBe(true)
    await t.pause()
    expect(t.paused.value).toBe(false)
    await t.toggle() // para
    expect(t.saved.value).toBe(b.path)
    expect(t.active.value).toBe(false)
    await t.start()
    expect(t.saved.value).toBe('')
  })
  it('modo área usa o seletor; cancelar não grava', async () => {
    const t = useToolbar()
    t.setMode('area')
    await t.start()
    expect(b.calls.some((c) => c[0] === 'start')).toBe(false)
    b.region = { x: 1, y: 2, w: 30, h: 40 }
    await t.start()
    expect((b.calls.find((c) => c[0] === 'start')![1] as { region: unknown }).region).toEqual(b.region)
  })
  it('atalho alterna gravação', async () => {
    const t = useToolbar()
    await t.init()
    b.emit('hotkey')
    await flush()
    expect(b.st.state).toBe('rec')
  })
  it('câmera, áudio do sistema e painel', async () => {
    const t = useToolbar()
    await t.setCam('c1')
    await t.setCam('')
    expect(b.calls.filter((c) => c[0] === 'toggleCam').map((c) => c[1])).toEqual([
      { visible: true, devId: 'c1' }, { visible: false, devId: undefined },
    ])
    t.toggleSys()
    expect(t.sys.value).toBe(false)
    t.toggleCfg()
    expect(t.cfg.value).toBe(true)
  })
  it('minimiza e fecha pela porta', async () => {
    const t = useToolbar()
    await t.minimize()
    await t.close()
    expect(b.calls.map((c) => c[0])).toEqual(['minimize', 'close'])
    b.fail = 'x'
    await t.close()
    expect(t.err.value).toBe('x')
  })
  it('a janela acompanha o conteúdo: linha extra e pílula', async () => {
    const t = useToolbar()
    await t.init()
    const fits = () => b.calls.filter((c) => c[0] === 'fitBar').map((c) => c[1])
    expect(t.extra.value).toBe(false) // um só monitor
    t.toggleCfg()
    await flush()
    expect(fits()).toEqual([{ w: 720, h: 184 }])
    t.toggleCfg()
    await t.toggle() // grava: pílula
    await flush()
    expect(fits().at(-1)).toEqual({ w: 260, h: 56 })
    b.fail = 'sem janela'
    t.toggleCfg()
    t.toggleCfg()
    t.rec.value = { state: 'idle', secs: 0 }
    await flush()
    expect(t.err.value).toBe('sem janela')
  })
  it('várias telas abrem a linha extra, exceto no modo área', async () => {
    b.tg.monitors.push({ id: 2, name: 'Tela 2', w: 1280, h: 720, x: 1920, y: 0, primary: false })
    const t = useToolbar()
    await t.init()
    expect(t.extra.value).toBe(true)
    t.setMode('area')
    expect(t.extra.value).toBe(false)
  })
})

describe('useWebcam', () => {
  it('abre a câmera da URL e mantém quadrado', async () => {
    history.replaceState(null, '', '/webcam.html?dev=c1')
    const w = useWebcam()
    await w.init()
    expect(m.calls).toEqual(['c1'])
    b.emit('resize', 100, 100)
    b.emit('resize', 100, 160)
    await flush()
    expect(b.calls.filter((c) => c[0] === 'fitSquare')).toEqual([['fitSquare', 160]])
    w.dispose()
    expect(m.stopped).toBe(1)
    expect(b.subs('resize')).toBe(0)
  })
  it('câmera padrão, erro e dispose sem stream', async () => {
    history.replaceState(null, '', '/webcam.html')
    m.fail = true
    const w = useWebcam()
    w.dispose() // antes do init: assinatura padrão vazia
    await w.init()
    expect(m.calls).toEqual([undefined])
    expect(w.err.value).toBe('Não foi possível abrir a câmera')
    const l = b.calls.find((c) => c[0] === 'logUi')![1] as { lvl: string; msg: string }
    expect(l.lvl).toBe('error')
    expect(l.msg).toContain('[webcam] getUserMedia falhou (dev=padrão)')
    expect(l.msg).toContain('sem câmera')
    w.dispose()
  })
  it('falha ao registrar a causa não atrapalha', async () => {
    history.replaceState(null, '', '/webcam.html?dev=c9')
    m.fail = true
    b.fail = 'sem log'
    const w = useWebcam()
    await w.init()
    expect(w.err.value).toBe('Não foi possível abrir a câmera')
    expect(b.calls.find((c) => c[0] === 'logUi')![1]).toMatchObject({ msg: expect.stringContaining('dev=c9') })
    w.dispose()
  })
})

describe('useRegion', () => {
  it('desenha, confirma com Enter e cancela com Esc', async () => {
    const r = useRegion()
    r.init()
    r.move(1, 1) // sem arrasto: ignora
    expect(r.shown.value).toBe(false)
    r.down(100, 100)
    r.move(300, 250)
    expect(r.shown.value).toBe(true)
    r.up()
    expect(r.css.value).toEqual({ x: 100, y: 100, w: 200, h: 150 })
    expect(r.tag.value).toBe('200 × 150')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'a' }))
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }))
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await flush()
    expect(b.calls.filter((c) => c[0] === 'submitRegion').map((c) => c[1])).toEqual([
      { x: 100, y: 100, w: 200, h: 150 }, null,
    ])
    r.dispose()
  })
  it('descarta seleção minúscula e Enter sem seleção não envia', async () => {
    const r = useRegion()
    r.down(10, 10)
    r.move(12, 12)
    r.up()
    expect(r.shown.value).toBe(false)
    r.key(new KeyboardEvent('keydown', { key: 'Enter' }))
    await flush()
    expect(b.calls).toEqual([])
  })
  it('mostra erro ao enviar', async () => {
    b.fail = 'sem janela'
    const r = useRegion()
    r.key(new KeyboardEvent('keydown', { key: 'Escape' }))
    await flush()
    expect(r.err.value).toBe('sem janela')
  })
})
