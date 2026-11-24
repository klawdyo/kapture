// Implementação REAL das portas: Tauri (invoke/listen) e getUserMedia.
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow, PhysicalSize } from '@tauri-apps/api/window'
import { pickCam } from '../domain/cam'
import type { Backend, Media } from './types'

/** Assina um evento Tauri entregando só o payload. */
const on = <T>(ev: string, cb: (p: T) => void) => listen<T>(ev, (e) => cb(e.payload))

export const realBackend: Backend = {
  targets: () => invoke('list_targets'),
  devices: () => invoke('list_devices'),
  toggleCam: (visible, devId) => invoke('toggle_cam', { visible, devId }),
  pickRegion: () => invoke('pick_region'),
  submitRegion: (region) => invoke('submit_region', { region }),
  start: (cfg) => invoke('start_rec', { cfg }),
  pause: () => invoke('pause_rec'),
  resume: () => invoke('resume_rec'),
  stop: () => invoke('stop_rec'),
  state: () => invoke('get_state'),
  micStart: (micId) => invoke('mic_start', { micId }),
  micStop: () => invoke('mic_stop'),
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
