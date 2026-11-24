// Lógica da janela da webcam: abre o stream e mantém o quadro 1:1.
import { shallowRef, ref } from 'vue'
import { fit } from '../domain/size'
import { ports } from '../ports'
import type { Off } from '../ports/types'

export function useWebcam() {
  const { backend: b, media } = ports()
  const stream = shallowRef<MediaStream | null>(null)
  const err = ref('')
  let off: Off = () => {}

  async function init(): Promise<void> {
    // O deviceId da câmera vem na URL da janela: webcam.html?dev=<id>
    const dev = new URLSearchParams(location.search).get('dev') ?? undefined
    try {
      stream.value = await media.openCam(dev)
    } catch {
      err.value = 'Não foi possível abrir a câmera'
    }
    off = await b.onResize((w, h) => {
      const s = fit(w, h)
      if (s !== null) void b.fitSquare(s)
    })
  }

  /** Libera a câmera e a assinatura de redimensionamento. */
  function dispose(): void {
    stream.value?.getTracks().forEach((t) => t.stop())
    stream.value = null
    off()
  }

  return { stream, err, init, dispose }
}
