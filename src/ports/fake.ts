// Implementação FAKE das portas, para testes (sem Tauri nem câmera).
import { pickCam, type VideoDev } from '../domain/cam'
import type { Devices, RecCfg, RecState, Region, Targets, Upd } from '../domain/types'
import type { Backend, LogLvl, Media, Off, Updates } from './types'

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
  /** Versão instalada simulada. */
  ver = '0.1.0'
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
  async micStart(micId: string) { await this.c('micStart', micId) }
  async micStop() { await this.c('micStop') }
  async logUi(lvl: LogLvl, msg: string) { await this.c('logUi', { lvl, msg }) }
  async openLogs() { await this.c('openLogs') }
  async version() { await this.c('version'); return this.ver }
  async openUrl(url: string) { await this.c('openUrl', url) }
  async minimize() { await this.c('minimize') }
  async close() { await this.c('close') }
  async fitBar(w: number, h: number) { await this.c('fitBar', { w, h }) }
  async fitSquare(size: number) { await this.c('fitSquare', size) }
  onMic(cb: (l: number) => void) { return this.on('mic', cb) }
  onState(cb: (s: RecState) => void) { return this.on('state', cb) }
  onHotkey(cb: () => void) { return this.on('hotkey', cb) }
  onResize(cb: (w: number, h: number) => void) { return this.on('resize', cb) }
}

export class FakeMedia implements Media {
  /** Câmeras que o "navegador" enumera. */
  cams: VideoDev[] = []
  /** Nomes pedidos. */
  calls: (string | undefined)[] = []
  /** deviceIds resolvidos (undefined = câmera padrão). */
  ids: (string | undefined)[] = []
  /** Quantas trilhas foram paradas. */
  stopped = 0
  /** Se definido, openCam falha. */
  fail = false

  async openCam(devName?: string): Promise<MediaStream> {
    this.calls.push(devName)
    if (this.fail) throw new Error('sem câmera')
    this.ids.push(pickCam(this.cams, devName))
    return { getTracks: () => [{ stop: () => { this.stopped++ } }] } as unknown as MediaStream
  }
}

export class FakeUpdates implements Updates {
  /** Resposta simulada de `check`. */
  res: Upd | null = null
  /** Versões atuais recebidas. */
  calls: string[] = []
  /** Se definido, `check` rejeita com este erro. */
  fail: unknown = null

  async check(current: string): Promise<Upd | null> {
    this.calls.push(current)
    if (this.fail !== null) throw this.fail
    return this.res
  }
}
