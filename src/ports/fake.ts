// Implementação FAKE das portas, para testes (sem Tauri nem câmera).
import type { Devices, RecCfg, RecState, Region, Targets } from '../domain/types'
import type { Backend, Media, Off } from './types'

/** Assinaturas de evento suportadas. */
interface Ev {
  mic: (l: number) => void
  state: (s: RecState) => void
  hotkey: () => void
  resize: (w: number, h: number) => void
}

export class FakeBackend implements Backend {
  /** Registro de chamadas: [comando, argumento]. */
  calls: [string, unknown?][] = []
  /** Se definido, todo comando falha com este erro. */
  fail: unknown = null
  tg: Targets = { monitors: [], wins: [] }
  dv: Devices = { cams: [], mics: [] }
  region: Region | null = null
  path = 'C:\\Videos\\kapture.mp4'
  st: RecState = { state: 'idle', secs: 0 }
  private ls: { [K in keyof Ev]: Set<Ev[K]> } = {
    mic: new Set(), state: new Set(), hotkey: new Set(), resize: new Set(),
  }

  /** Registra a chamada e simula falha quando pedido. */
  private async c(name: string, arg?: unknown): Promise<void> {
    this.calls.push([name, arg])
    if (this.fail !== null) throw this.fail
  }

  private on<K extends keyof Ev>(k: K, cb: Ev[K]): Promise<Off> {
    this.ls[k].add(cb)
    return Promise.resolve(() => { this.ls[k].delete(cb) })
  }

  /** Dispara um evento para os assinantes (uso dos testes). */
  emit<K extends keyof Ev>(k: K, ...a: Parameters<Ev[K]>): void {
    this.ls[k].forEach((f) => (f as (...x: unknown[]) => void)(...a))
  }

  /** Quantidade de assinantes de um evento. */
  subs(k: keyof Ev): number {
    return this.ls[k].size
  }

  async targets() { await this.c('targets'); return this.tg }
  async devices() { await this.c('devices'); return this.dv }
  async toggleCam(visible: boolean, devId?: string) { await this.c('toggleCam', { visible, devId }) }
  async pickRegion() { await this.c('pickRegion'); return this.region }
  async submitRegion(region: Region | null) { await this.c('submitRegion', region) }
  async start(cfg: RecCfg) { await this.c('start', cfg); this.st = { state: 'rec', secs: 0 } }
  async pause() { await this.c('pause'); this.st = { state: 'paused', secs: this.st.secs } }
  async resume() { await this.c('resume'); this.st = { state: 'rec', secs: this.st.secs } }
  async stop() { await this.c('stop'); this.st = { state: 'idle', secs: 0 }; return this.path }
  async state() { await this.c('state'); return this.st }
  async fitSquare(size: number) { await this.c('fitSquare', size) }
  onMic(cb: (l: number) => void) { return this.on('mic', cb) }
  onState(cb: (s: RecState) => void) { return this.on('state', cb) }
  onHotkey(cb: () => void) { return this.on('hotkey', cb) }
  onResize(cb: (w: number, h: number) => void) { return this.on('resize', cb) }
}

export class FakeMedia implements Media {
  /** deviceIds pedidos. */
  calls: (string | undefined)[] = []
  /** Quantas trilhas foram paradas. */
  stopped = 0
  /** Se definido, openCam falha. */
  fail = false

  async openCam(devId?: string): Promise<MediaStream> {
    this.calls.push(devId)
    if (this.fail) throw new Error('sem câmera')
    return { getTracks: () => [{ stop: () => { this.stopped++ } }] } as unknown as MediaStream
  }
}
