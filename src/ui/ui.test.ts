// Testes dos widgets genéricos e especializados.
import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import DevSelect from '../components/DevSelect.vue'
import ModeSwitch from '../components/ModeSwitch.vue'
import RecBtn from '../components/RecBtn.vue'
import TitleBar from '../components/TitleBar.vue'
import VuMeter from '../components/VuMeter.vue'
import Box from './Box.vue'
import Btn from './Btn.vue'
import Icon from './Icon.vue'
import Meter from './Meter.vue'
import Segmented from './Segmented.vue'
import Select from './Select.vue'
import Video from './Video.vue'

describe('ui genérico', () => {
  it('Icon: contorno e preenchido', () => {
    expect(mount(Icon, { props: { name: 'cam' } }).attributes('fill')).toBe('none')
    const s = mount(Icon, { props: { name: 'rec', size: 30 } })
    expect(s.attributes('fill')).toBe('currentColor')
    expect(s.attributes('width')).toBe('30')
  })
  it('Box: tag configurável e slot', () => {
    expect(mount(Box, { slots: { default: 'x' } }).element.tagName).toBe('DIV')
    expect(mount(Box, { props: { tag: 'span' } }).element.tagName).toBe('SPAN')
  })
  it('Btn: ícone opcional, estados e clique', async () => {
    const a = mount(Btn, { slots: { default: 'Oi' } })
    expect(a.find('svg').exists()).toBe(false)
    expect(a.text()).toBe('Oi')
    const b = mount(Btn, { props: { icon: 'gear', on: true, round: true, kind: 'rec', col: true } })
    expect(b.find('svg').exists()).toBe(true)
    expect(b.classes()).toEqual(expect.arrayContaining(['on', 'round', 'rec']))
    await b.trigger('click')
    expect(b.emitted('click')).toHaveLength(1)
  })
  it('Select: dica, seta e v-model', async () => {
    const s = mount(Select, { props: { modelValue: 'a', options: [{ value: 'a', label: 'A' }, { value: 'b', label: 'B' }] } })
    expect(s.find('select').attributes('title')).toBe('A') // dica com o nome completo
    await s.find('select').setValue('b')
    expect(s.emitted('update:modelValue')![0]).toEqual(['b'])
    expect(mount(Select, { props: { modelValue: 'x', options: [], title: 'T' } }).find('select').attributes('title')).toBe('')
  })
  it('Segmented: marca e emite', async () => {
    const s = mount(Segmented, { props: { modelValue: 'a', options: [{ value: 'a', label: 'A' }, { value: 'b', label: 'B', icon: 'area' }] } })
    expect(s.findAll('button')[0].classes()).toContain('on')
    await s.findAll('button')[1].trigger('click')
    expect(s.emitted('update:modelValue')![0]).toEqual(['b'])
  })
  it('Meter: acende segmentos e passa para vermelho', () => {
    const lo = mount(Meter, { props: { value: 0.3 } })
    expect(lo.findAll('i.on')).toHaveLength(3)
    expect(lo.findAll('i.hot')).toHaveLength(3) // classe hot marca os segmentos altos
    const hi = mount(Meter, { props: { value: 1, segs: 4, hot: 0.5 } })
    expect(hi.findAll('i.on.hot')).toHaveLength(2)
  })
  it('Video: liga o stream e reage a mudanças', async () => {
    const s1 = {} as MediaStream
    const v = mount(Video, { props: { src: s1, mirror: true } })
    const el = v.element as HTMLVideoElement
    expect(el.srcObject).toBe(s1)
    expect(v.classes()).toContain('mirror')
    const s2 = {} as MediaStream
    await v.setProps({ src: s2 })
    expect(el.srcObject).toBe(s2)
    expect(mount(Video, { props: { src: null } }).classes()).not.toContain('mirror')
  })
})

describe('components do domínio', () => {
  it('ModeSwitch emite o modo', async () => {
    const m = mount(ModeSwitch, { props: { modelValue: 'screen' } })
    expect(m.text()).toContain('Janela')
    await m.findAll('button')[2].trigger('click')
    expect(m.emitted('update:modelValue')![0]).toEqual(['area'])
  })
  it('DevSelect inclui "nenhum" e emite', async () => {
    const d = mount(DevSelect, { props: { modelValue: '', devs: [{ id: 'x', name: 'Mic X' }], icon: 'mic', label: 'Microfone', none: 'Sem mic' }, slots: { default: '<b class="s">vu</b>' } })
    expect(d.text()).toContain('Microfone')
    expect(d.find('.s').exists()).toBe(true)
    expect(d.findAll('option').map((o) => o.text())).toEqual(['Sem mic', 'Mic X'])
    await d.find('select').setValue('x')
    expect(d.emitted('update:modelValue')![0]).toEqual(['x'])
  })
  it('VuMeter repassa o nível', () => {
    expect(mount(VuMeter, { props: { value: 0.5 } }).findAll('i.on')).toHaveLength(4)
  })
  it('RecBtn alterna ícone e título', () => {
    expect(mount(RecBtn, { props: { rec: false } }).attributes('title')).toBe('Gravar')
    expect(mount(RecBtn, { props: { rec: true } }).attributes('title')).toBe('Parar')
    expect(mount(RecBtn, { props: { rec: false } }).text()).toContain('Gravar')
  })
  it('TitleBar emite minimizar e fechar', async () => {
    const t = mount(TitleBar, { slots: { default: 'status' } })
    expect(t.text()).toContain('Gravador de Tela')
    expect(t.text()).toContain('status')
    await t.find('button[title="Minimizar"]').trigger('click')
    await t.find('button[title="Fechar"]').trigger('click')
    expect(t.emitted('min')).toHaveLength(1)
    expect(t.emitted('close')).toHaveLength(1)
  })
})
