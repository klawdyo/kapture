// Implementação REAL das portas: Tauri (invoke/listen) e getUserMedia.
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow, PhysicalSize } from '@tauri-apps/api/window'
import { pickCam } from '../domain/cam'
import { detail } from '../domain/err'
import type { Backend, Media } from './types'

/** Assina um evento Tauri entregando só o payload. */
const on = <T>(ev: string, cb: (p: T) => void) => listen<T>(ev, (e) => cb(e.payload))

/** Envia uma mensagem ao log em disco (falhas aqui são descartadas). */
const logUi = (lvl: string, msg: string): Promise<void> =>
  invoke<void>('log_ui', { lvl, msg }).catch(() => {})

/**
 * Chama um comando; se falhar, registra a falha no log do backend e
 * repassa o erro. Pega também o que o backend nunca viu (comando
 * inexistente, permissão negada, argumento inválido).
 */
const call = <T>(cmd: string, args?: Record<string, unknown>): Promise<T> =>
  (args ? invoke<T>(cmd, args) : invoke<T>(cmd)).catch((e: unknown) => {
    void logUi('error', `[ipc] ${cmd} falhou: ${detail(e)}`)
    throw e
  })

export const realBackend: Backend = {
  targets: () => call('list_targets'),
  devices: () => call('list_devices'),
  toggleCam: (visible, devId) => call('toggle_cam', { visible, devId }),
  pickRegion: () => call('pick_region'),
  submitRegion: (region) => call('submit_region', { region }),
  start: (cfg) => call('start_rec', { cfg }),
  pause: () => call('pause_rec'),
  resume: () => call('resume_rec'),
  stop: () => call('stop_rec'),
  state: () => call('get_state'),
  micStart: (micId) => call('mic_start', { micId }),
  micStop: () => call('mic_stop'),
  logUi,
  openLogs: () => call('open_logs'),
  fitSquare: (size) => getCurrentWindow().setSize(new PhysicalSize(size, size)),
  onMic: (cb) => on<{ level: number }>('mic-level', (p) => cb(p.level)),
  onState: (cb) => on('rec-state', cb),
  onHotkey: (cb) => on('hotkey', () => cb()),
  onResize: (cb) => getCurrentWindow().onResized((e) => cb(e.payload.width, e.payload.height)),
}

/** Câmeras (videoinput) visíveis; labels vêm vazios até haver permissão. */
const cams = async (): Promise<MediaDeviceInfo[]> =>
  (await navigator.mediaDevices.enumerateDevices()).filter((d) => d.kind === 'videoinput')

export const realMedia: Media = {
  // `dev` é o nome amigável do backend; o deviceId do WebView2 é um hash,
  // então resolvemos pelo label. Sem casamento usa a câmera padrão.
  openCam: async (dev) => {
    const md = navigator.mediaDevices
    let list = dev ? await cams() : []
    if (dev && list.every((d) => d.label === '')) {
      // Um getUserMedia prévio libera os labels.
      const tmp = await md.getUserMedia({ video: true, audio: false })
      tmp.getTracks().forEach((t) => t.stop())
      list = await cams()
    }
    const id = pickCam(list, dev)
    return md.getUserMedia({ video: id ? { deviceId: { exact: id } } : true, audio: false })
  },
}
