// Testes da inicialização das janelas e do envio de erros ao log.
import { flushPromises } from '@vue/test-utils'
import { beforeEach, describe, expect, it } from 'vitest'
import { defineComponent, h } from 'vue'
import { boot, installErrorLog, MAX_LOGS } from './boot'
import { setPorts } from './ports'
import { FakeBackend, FakeMedia } from './ports/fake'

let b: FakeBackend

beforeEach(() => {
  b = new FakeBackend()
  setPorts({ backend: b, media: new FakeMedia() })
})

const logs = () => b.calls.filter((c) => c[0] === 'logUi').map((c) => c[1] as { lvl: string; msg: string })

describe('installErrorLog', () => {
  it('envia erros não tratados e promessas rejeitadas', () => {
    const t = new EventTarget()
    installErrorLog('toolbar', t)
    t.dispatchEvent(new ErrorEvent('error', { message: 'boom', filename: 'a.js', lineno: 1, colno: 2 }))
    t.dispatchEvent(Object.assign(new Event('unhandledrejection'), { reason: 'nope' }))
    expect(logs()).toEqual([
      { lvl: 'error', msg: '[toolbar] erro não tratado: boom em a.js:1:2' },
      { lvl: 'error', msg: '[toolbar] promessa rejeitada: nope' },
    ])
  })
  it('limita a quantidade de mensagens', () => {
    const t = new EventTarget()
    const send = installErrorLog('x', t)
    for (let i = 0; i < MAX_LOGS + 5; i++) send(`m${i}`)
    const l = logs()
    expect(l).toHaveLength(MAX_LOGS + 1)
    expect(l[MAX_LOGS - 1].msg).toBe(`[x] m${MAX_LOGS - 1}`)
    expect(l[MAX_LOGS].msg).toContain('limite de mensagens')
  })
  it('falha ao enviar ao backend é ignorada', async () => {
    const t = new EventTarget()
    const send = installErrorLog('x', t)
    b.fail = 'sem backend'
    send('m')
    await flushPromises()
    expect(logs()).toHaveLength(1)
  })
})

describe('boot', () => {
  it('monta o app e registra erros de componentes', async () => {
    document.body.innerHTML = '<div id="app"></div>'
    const Ruim = defineComponent({
      setup() {
        throw new Error('quebrou no setup')
      },
      render: () => h('div'),
    })
    boot(Ruim, 'webcam')
    await flushPromises()
    const l = logs()
    expect(l).toHaveLength(1)
    expect(l[0].msg).toContain('[webcam] erro no componente (setup function)')
    expect(l[0].msg).toContain('quebrou no setup')
  })
})
