// PORTAS (interfaces) — única fronteira entre a UI e o mundo externo.
import type { Devices, RecCfg, RecState, Region, Targets } from '../domain/types'

/** Função que cancela uma assinatura de evento. */
export type Off = () => void

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
  /** Força a janela atual a ficar quadrada (px físicos). */
  fitSquare(size: number): Promise<void>
  onMic(cb: (level: number) => void): Promise<Off>
  onState(cb: (s: RecState) => void): Promise<Off>
  onHotkey(cb: () => void): Promise<Off>
  onResize(cb: (w: number, h: number) => void): Promise<Off>
}

/** Porta de mídia do navegador (câmera). */
export interface Media {
  /** `devName` é o nome amigável da câmera (id do backend). */
  openCam(devName?: string): Promise<MediaStream>
}

/** Conjunto de portas injetável. */
export interface Ports { backend: Backend; media: Media }
