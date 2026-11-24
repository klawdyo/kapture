// PORTAS (interfaces) — única fronteira entre a UI e o mundo externo.
import type { Devices, RecCfg, RecState, Region, Targets, Upd, UpdSt } from '../domain/types'

/** Função que cancela uma assinatura de evento. */
export type Off = () => void

/** Nível das mensagens enviadas ao log em disco. */
export type LogLvl = 'debug' | 'info' | 'warn' | 'error'

/** Porta do backend Tauri (comandos + eventos). */
export interface Backend {
  targets(): Promise<Targets>
  devices(): Promise<Devices>
  toggleCam(visible: boolean, devId?: string): Promise<void>
  pickRegion(): Promise<Region | null>
  submitRegion(region: Region | null): Promise<void>
  start(cfg: RecCfg): Promise<void>
  pause(): Promise<void>
  resume(): Promise<void>
  stop(): Promise<string>
  state(): Promise<RecState>
  /** Liga o VU meter do microfone (reinicia sem erro se já ligado). */
  micStart(micId: string): Promise<void>
  /** Desliga o VU meter. */
  micStop(): Promise<void>
  /** Grava uma mensagem da interface no log em disco do backend. */
  logUi(lvl: LogLvl, msg: string): Promise<void>
  /** Abre a pasta de logs no Explorer. */
  openLogs(): Promise<void>
  /** Versão instalada do app (ex.: '0.1.0'). */
  version(): Promise<string>
  /** Abre no navegador uma página de release (o backend recusa outras URLs). */
  openUrl(url: string): Promise<void>
  /** Inicia o ciclo de atualização no backend (uma vez; o resultado chega por `onUpdate`). */
  updateStart(): Promise<void>
  /** Reinicia o app aplicando a atualização já baixada. */
  updateApply(): Promise<void>
  /** Minimiza a janela atual. */
  minimize(): Promise<void>
  /** Fecha a janela atual (a da barra encerra o app). */
  close(): Promise<void>
  /** Redimensiona a barra (px lógicos), mantendo o centro da base. */
  fitBar(w: number, h: number): Promise<void>
  /** Força a janela atual a ficar quadrada (px físicos). */
  fitSquare(size: number): Promise<void>
  onMic(cb: (level: number) => void): Promise<Off>
  onState(cb: (s: RecState) => void): Promise<Off>
  onUpdate(cb: (s: UpdSt) => void): Promise<Off>
  onHotkey(cb: () => void): Promise<Off>
  onResize(cb: (w: number, h: number) => void): Promise<Off>
}

/** Porta de mídia do navegador (câmera). */
export interface Media {
  /** `devName` é o nome amigável da câmera (id do backend). */
  openCam(devName?: string): Promise<MediaStream>
}

/** Porta de atualizações (Releases do GitHub). */
export interface Updates {
  /** Atualização mais nova que `current`, ou null se não houver. Rejeita em falha de rede. */
  check(current: string): Promise<Upd | null>
}

/** Conjunto de portas injetável. */
export interface Ports { backend: Backend; media: Media; updates: Updates }
