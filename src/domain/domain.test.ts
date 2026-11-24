// Testes da lógica pura de domínio.
import { describe, expect, it } from 'vitest'
import { buildCfg, FPS, targetOpts } from './cfg'
import { msg } from './err'
import { idle, isActive, isPaused } from './rec'
import { big, label, MIN, norm, phys } from './rect'
import { fit } from './size'
import { fmt } from './time'

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
