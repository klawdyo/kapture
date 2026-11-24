// Tipos do contrato IPC (JSON camelCase) — espelham docs/CONTRACT.md.

/** Retângulo em pixels. */
export interface Region { x: number; y: number; w: number; h: number }
/** Modo de captura. */
export type Mode = 'screen' | 'window' | 'area'
/** Monitor disponível. */
export interface Monitor { id: number; name: string; w: number; h: number; x: number; y: number; primary: boolean }
/** Janela capturável. */
export interface Win { id: number; title: string }
/** Alvos de captura. */
export interface Targets { monitors: Monitor[]; wins: Win[] }
/** Dispositivo de mídia (câmera ou microfone). */
export interface Dev { id: string; name: string }
/** Dispositivos disponíveis. */
export interface Devices { cams: Dev[]; mics: Dev[] }
/** Configuração de gravação. */
export interface RecCfg {
  mode: Mode
  targetId?: number
  region?: Region
  micId?: string
  sys: boolean
  fps: number
  dir?: string
}
/** Estados possíveis da gravação. */
export type RecKind = 'idle' | 'rec' | 'paused'
/** Estado da gravação + segundos decorridos. */
export interface RecState { state: RecKind; secs: number }
/** Opção genérica de lista (valor + rótulo). */
export interface Opt { value: string; label: string }
