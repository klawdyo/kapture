// Testes da lógica pura de domínio.
import { describe, expect, it } from 'vitest'
import { buildCfg, FPS, targetOpts } from './cfg'
import { pickCam } from './cam'
import { detail, msg } from './err'
import { fmtError, fmtReject, fmtVue } from './log'
import { idle, isActive, isPaused } from './rec'
import { big, cursor, HANDLES, hit, label, MIN, norm, org, phys, pin, resize, room, shift } from './rect'
import { BAR, barSize, EXTRA_H, fit, PILL } from './size'
import { fmt } from './time'

describe('rect: alças, mover e redimensionar', () => {
  const r = { x: 100, y: 100, w: 200, h: 100 }
  const b = { w: 800, h: 600 }
  it('hit-test: cantos, meios, interior e fora', () => {
    expect(hit(r, { x: 101, y: 99 })).toBe('nw')
    expect(hit(r, { x: 299, y: 99 })).toBe('ne')
    expect(hit(r, { x: 101, y: 201 })).toBe('sw')
    expect(hit(r, { x: 301, y: 201 })).toBe('se')
    expect(hit(r, { x: 200, y: 98 })).toBe('n')
    expect(hit(r, { x: 190, y: 202 })).toBe('s')
    expect(hit(r, { x: 98, y: 150 })).toBe('w')
    expect(hit(r, { x: 302, y: 140 })).toBe('e')
    expect(hit(r, { x: 150, y: 120 })).toBe('move')
    expect(hit(r, { x: 150, y: 101 })).toBe('move') // perto do topo, fora do meio
    expect(hit(r, { x: 150, y: 150 })).toBe('move')
    expect(hit(r, { x: 500, y: 500 })).toBe(null)
    expect(hit(r, { x: 150, y: 80 })).toBe(null)
    expect(hit(r, { x: 80, y: 120 })).toBe(null)
    expect(hit(r, { x: 150, y: 300 })).toBe(null)
    expect(HANDLES).toHaveLength(8)
  })
  it('cursor por alvo', () => {
    expect(cursor(null)).toBe('crosshair')
    expect(cursor('move')).toBe('move')
    expect(cursor('n')).toBe('ns-resize')
    expect(cursor('s')).toBe('ns-resize')
    expect(cursor('e')).toBe('ew-resize')
    expect(cursor('w')).toBe('ew-resize')
    expect(cursor('nw')).toBe('nwse-resize')
    expect(cursor('se')).toBe('nwse-resize')
    expect(cursor('ne')).toBe('nesw-resize')
    expect(cursor('sw')).toBe('nesw-resize')
  })
  it('prende ponto e move dentro da tela', () => {
    expect(pin({ x: -5, y: 900 }, b)).toEqual({ x: 0, y: 600 })
    expect(pin({ x: 5, y: 6 }, b)).toEqual({ x: 5, y: 6 })
    expect(shift(r, 10, -20, b)).toEqual({ x: 110, y: 80, w: 200, h: 100 })
    expect(shift(r, -500, -500, b)).toEqual({ x: 0, y: 0, w: 200, h: 100 })
    expect(shift(r, 900, 900, b)).toEqual({ x: 600, y: 500, w: 200, h: 100 })
  })
  it('redimensiona por cada alça, com mínimo e limites', () => {
    expect(resize(r, 'e', 20, 99, b)).toEqual({ x: 100, y: 100, w: 220, h: 100 })
    expect(resize(r, 'w', 20, 99, b)).toEqual({ x: 120, y: 100, w: 180, h: 100 })
    expect(resize(r, 'n', 99, 20, b)).toEqual({ x: 100, y: 120, w: 200, h: 80 })
    expect(resize(r, 's', 99, 20, b)).toEqual({ x: 100, y: 100, w: 200, h: 120 })
    expect(resize(r, 'nw', -10, -10, b)).toEqual({ x: 90, y: 90, w: 210, h: 110 })
    expect(resize(r, 'ne', 10, -10, b)).toEqual({ x: 100, y: 90, w: 210, h: 110 })
    expect(resize(r, 'sw', -10, 10, b)).toEqual({ x: 90, y: 100, w: 210, h: 110 })
    expect(resize(r, 'se', 10, 10, b)).toEqual({ x: 100, y: 100, w: 210, h: 110 })
    // mínimo 16x16 (sem inverter o lado)
    expect(resize(r, 'e', -500, 0, b)).toEqual({ x: 100, y: 100, w: MIN, h: 100 })
    expect(resize(r, 'w', 500, 0, b)).toEqual({ x: 284, y: 100, w: MIN, h: 100 })
    expect(resize(r, 's', 0, -500, b)).toEqual({ x: 100, y: 100, w: 200, h: MIN })
    expect(resize(r, 'n', 0, 500, b)).toEqual({ x: 100, y: 184, w: 200, h: MIN })
    // limites da tela
    expect(resize(r, 'nw', -500, -500, b)).toEqual({ x: 0, y: 0, w: 300, h: 200 })
    expect(resize(r, 'se', 900, 900, b)).toEqual({ x: 100, y: 100, w: 700, h: 500 })
  })
  it('etiqueta cabe abaixo?', () => {
    expect(room(r, b)).toBe(true)
    expect(room({ x: 0, y: 0, w: 10, h: 580 }, b)).toBe(false)
  })
})

describe('rect', () => {
  it('normaliza arrasto em qualquer direção', () => {
    expect(norm({ x: 10, y: 20 }, { x: 40, y: 50 })).toEqual({ x: 10, y: 20, w: 30, h: 30 })
    expect(norm({ x: 40, y: 50 }, { x: 10, y: 20 })).toEqual({ x: 10, y: 20, w: 30, h: 30 })
  })
  it('valida tamanho mínimo', () => {
    expect(big({ x: 0, y: 0, w: MIN, h: MIN })).toBe(true)
    expect(big({ x: 0, y: 0, w: MIN - 1, h: 100 })).toBe(false)
    expect(big({ x: 0, y: 0, w: 100, h: MIN - 1 })).toBe(false)
  })
  it('converte para px físicos', () => {
    expect(phys({ x: 10, y: 5, w: 100, h: 50 }, 1.5, 100, 0)).toEqual({ x: 115, y: 8, w: 150, h: 75 })
    // monitor à esquerda do principal (origem negativa) e escala própria
    expect(phys({ x: 10, y: 5, w: 100, h: 50 }, 1.25, -1920, 0)).toEqual({ x: -1907, y: 6, w: 125, h: 63 })
    // monitor 4K (escala 2) abaixo do principal
    expect(phys({ x: 0, y: 0, w: 640, h: 360 }, 2, 0, 1080)).toEqual({ x: 0, y: 1080, w: 1280, h: 720 })
  })
  it('gera etiqueta', () => {
    expect(label({ x: 0, y: 0, w: 1280, h: 720 })).toBe('1280 × 720')
  })
})

describe('time', () => {
  it('formata mm:ss e h:mm:ss', () => {
    expect(fmt(0)).toBe('00:00')
    expect(fmt(65.9)).toBe('01:05')
    expect(fmt(3600)).toBe('1:00:00')
    expect(fmt(3725)).toBe('1:02:05')
    expect(fmt(-5)).toBe('00:00')
  })
})

describe('rec', () => {
  it('classifica estados', () => {
    expect(isActive(idle)).toBe(false)
    expect(isActive({ state: 'rec', secs: 1 })).toBe(true)
    expect(isPaused({ state: 'paused', secs: 1 })).toBe(true)
    expect(isPaused(idle)).toBe(false)
  })
})

describe('cfg', () => {
  const base = { mode: 'screen', tid: '', mic: '', sys: true, fps: 30 } as const
  it('omite campos vazios', () => {
    expect(buildCfg(base)).toEqual({ mode: 'screen', sys: true, fps: 30 })
  })
  it('inclui alvo, mic e região', () => {
    const region = { x: 1, y: 2, w: 3, h: 4 }
    expect(buildCfg({ ...base, tid: '7', mic: 'm1', region })).toEqual({
      mode: 'screen', sys: true, fps: 30, targetId: 7, micId: 'm1', region,
    })
  })
  it('área ignora alvo', () => {
    expect(buildCfg({ ...base, mode: 'area', tid: '7' }).targetId).toBeUndefined()
  })
  it('lista fps', () => {
    expect(FPS.map((o) => o.value)).toEqual(['15', '24', '30', '60'])
  })
  it('opções de alvo por modo', () => {
    const t = {
      monitors: [{ id: 1, name: 'Tela 1', w: 1920, h: 1080, x: 0, y: 0, primary: true }],
      wins: [{ id: 9, title: 'Bloco de notas' }],
    }
    expect(targetOpts('screen', t)).toEqual([{ value: '1', label: 'Tela 1 (1920×1080)' }])
    expect(targetOpts('window', t)).toEqual([{ value: '9', label: 'Bloco de notas' }])
    expect(targetOpts('area', t)).toEqual([])
  })
})

describe('err e size', () => {
  it('extrai mensagem', () => {
    expect(msg('falhou')).toBe('falhou')
    expect(msg(new Error('x'))).toBe('x')
    expect(msg(42)).toBe('Erro desconhecido')
  })
  it('ajusta quadro 1:1', () => {
    expect(fit(100, 100)).toBeNull()
    expect(fit(100, 150)).toBe(150)
    expect(fit(200, 150)).toBe(200)
  })
})

describe('cam', () => {
  const l = [
    { deviceId: 'a', label: 'USB Camera' },
    { deviceId: 'b', label: 'Logitech C920 HD Pro' },
    { deviceId: 'c', label: 'Logitech C920' },
    { deviceId: 'd', label: '' },
  ]
  it('prefere igualdade, ignorando caixa e espaços', () => {
    expect(pickCam(l, ' logitech c920 ')).toBe('c')
  })
  it('aceita "contém" nos dois sentidos', () => {
    expect(pickCam(l, 'C920 HD')).toBe('b')
    expect(pickCam(l, 'Integrated USB Camera (04f2:b6d0)')).toBe('a')
    expect(pickCam([{ deviceId: 'a', label: 'USB Camera' }], 'USB Camera (04f2:b6d0)')).toBe('a')
  })
  it('sem nome, sem casamento ou label vazio: padrão', () => {
    expect(pickCam(l)).toBeUndefined()
    expect(pickCam(l, '  ')).toBeUndefined()
    expect(pickCam(l, 'nada')).toBeUndefined()
    expect(pickCam([{ deviceId: 'd', label: '' }], 'x')).toBeUndefined()
  })
})

describe('erros no log', () => {
  it('detail: pilha, texto, JSON e valores estranhos', () => {
    const e = new Error('x')
    expect(detail(e)).toBe(e.stack)
    const sem = new Error('y')
    sem.stack = undefined
    expect(detail(sem)).toBe('Error: y')
    expect(detail('texto')).toBe('texto')
    expect(detail({ a: 1 })).toBe('{"a":1}')
    expect(detail(undefined)).toBe('undefined')
    const ciclo: Record<string, unknown> = {}
    ciclo.self = ciclo
    expect(detail(ciclo)).toBe('[object Object]')
  })
  it('fmtError: mensagem, local e pilha', () => {
    const e = new Error('boom')
    e.stack = 'Error: boom\n  at a.js:1:1'
    expect(fmtError({ message: 'boom', filename: 'a.js', lineno: 3, colno: 4, error: e })).toBe(
      'erro não tratado: boom em a.js:3:4\nError: boom\n  at a.js:1:1',
    )
    expect(fmtError({ message: 'x', filename: 'b.js' })).toBe('erro não tratado: x em b.js:0:0')
    expect(fmtError({ message: 'y', error: 'texto' })).toBe('erro não tratado: y')
    const sem = new Error('z')
    sem.stack = ''
    expect(fmtError({ message: 'z', error: sem })).toBe('erro não tratado: z')
  })
  it('fmtReject e fmtVue', () => {
    expect(fmtReject('nope')).toBe('promessa rejeitada: nope')
    expect(fmtVue('x', 'render')).toBe('erro no componente (render): x')
  })
})

describe('barSize', () => {
  it('escolhe o tamanho da barra', () => {
    expect(barSize(false, false)).toEqual(BAR)
    expect(barSize(false, true)).toEqual({ w: BAR.w, h: BAR.h + EXTRA_H })
    expect(barSize(true, true)).toEqual(PILL)
  })
})

describe('org (query string do seletor)', () => {
  it('lê origem e escala, inclusive negativas e fracionárias', () => {
    expect(org('?ox=-1920&oy=-200&s=1.25')).toEqual({ ox: -1920, oy: -200, s: 1.25 })
    expect(org('?ox=0&oy=1080&s=2')).toEqual({ ox: 0, oy: 1080, s: 2 })
  })
  it('ausente, vazio, inválido ou escala não positiva vira o padrão', () => {
    const d = { ox: 0, oy: 0, s: 1 }
    expect(org('')).toEqual(d)
    expect(org('?ox=&oy=abc&s=')).toEqual(d)
    expect(org('?s=0')).toEqual(d)
    expect(org('?s=-2')).toEqual(d)
  })
})
