// Implementação REAL das portas: Tauri (invoke/listen) e getUserMedia.
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow, PhysicalSize } from '@tauri-apps/api/window'
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
  fitSquare: (size) => getCurrentWindow().setSize(new PhysicalSize(size, size)),
  onMic: (cb) => on<{ level: number }>('mic-level', (p) => cb(p.level)),
  onState: (cb) => on('rec-state', cb),
  onHotkey: (cb) => on('hotkey', () => cb()),
  onResize: (cb) => getCurrentWindow().onResized((e) => cb(e.payload.width, e.payload.height)),
}

export const realMedia: Media = {
  // Sem deviceId usa a câmera padrão; com deviceId exige aquela câmera.
  openCam: (devId) =>
    navigator.mediaDevices.getUserMedia({
      video: devId ? { deviceId: { exact: devId } } : true,
      audio: false,
    }),
}
