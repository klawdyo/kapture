//! Captura de áudio com WASAPI (modo compartilhado, por polling):
//! microfone e loopback do sistema. O WASAPI converte para 48 kHz,
//! 16 bits, estéreo (`AUTOCONVERTPCM`), então não há reamostragem no app.

use super::super::com::wide;
use cap_core::media::{CH, RATE};
use cap_core::{CapErr, Res};
use windows::core::PCWSTR;
use windows::Win32::Media::Audio::{
    eConsole, eRender, IAudioCaptureClient, IAudioClient, IMMDevice, IMMDeviceEnumerator,
    MMDeviceEnumerator, AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED,
    AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM, AUDCLNT_STREAMFLAGS_LOOPBACK,
    AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY, WAVEFORMATEX, WAVE_FORMAT_PCM,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

/// Leitor de uma fonte de áudio.
pub struct Reader {
    client: IAudioClient,
    cap: IAudioCaptureClient,
}

fn fail(e: windows::core::Error) -> CapErr {
    CapErr::from(format!("Falha ao abrir o áudio: {}", e.message()))
}

impl Reader {
    /// Microfone pelo id do endpoint.
    pub fn mic(id: &str) -> Res<Reader> {
        let en = enumerator()?;
        let w = wide(id);
        // SAFETY: `w` é terminado em nulo e vive durante a chamada.
        let dev = unsafe { en.GetDevice(PCWSTR(w.as_ptr())) }
            .map_err(|_| CapErr::from("O microfone escolhido não está disponível."))?;
        Reader::open(&dev, false)
    }

    /// Som do sistema (loopback do dispositivo de saída padrão).
    pub fn system() -> Res<Reader> {
        let en = enumerator()?;
        // SAFETY: consulta simples ao enumerador.
        let dev = unsafe { en.GetDefaultAudioEndpoint(eRender, eConsole) }
            .map_err(|_| CapErr::from("Nenhuma saída de áudio padrão foi encontrada."))?;
        Reader::open(&dev, true)
    }

    fn open(dev: &IMMDevice, loopback: bool) -> Res<Reader> {
        let fmt = WAVEFORMATEX {
            wFormatTag: WAVE_FORMAT_PCM as u16,
            nChannels: CH as u16,
            nSamplesPerSec: RATE,
            nAvgBytesPerSec: RATE * CH as u32 * 2,
            nBlockAlign: CH as u16 * 2,
            wBitsPerSample: 16,
            cbSize: 0,
        };
        let mut flags =
            AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY;
        if loopback {
            flags |= AUDCLNT_STREAMFLAGS_LOOPBACK;
        }
        // SAFETY: sequência padrão de inicialização do WASAPI; `fmt` vive
        // durante `Initialize` (que copia o formato).
        unsafe {
            let client: IAudioClient = dev.Activate(CLSCTX_ALL, None).map_err(fail)?;
            // Buffer de 200 ms (unidades de 100 ns).
            client
                .Initialize(AUDCLNT_SHAREMODE_SHARED, flags, 2_000_000, 0, &fmt, None)
                .map_err(fail)?;
            let cap: IAudioCaptureClient = client.GetService().map_err(fail)?;
            client.Start().map_err(fail)?;
            Ok(Reader { client, cap })
        }
    }

    /// Acrescenta a `out` todas as amostras disponíveis agora.
    pub fn read(&self, out: &mut Vec<i16>) {
        // SAFETY: cada GetBuffer é pareado com ReleaseBuffer; o slice cobre
        // exatamente `frames * canais` amostras i16 do buffer devolvido.
        unsafe {
            while let Ok(n) = self.cap.GetNextPacketSize() {
                if n == 0 {
                    break;
                }
                let (mut p, mut frames, mut flags) = (std::ptr::null_mut(), 0u32, 0u32);
                if self
                    .cap
                    .GetBuffer(&mut p, &mut frames, &mut flags, None, None)
                    .is_err()
                {
                    break;
                }
                let len = frames as usize * CH;
                if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 || p.is_null() {
                    out.resize(out.len() + len, 0);
                } else {
                    out.extend_from_slice(std::slice::from_raw_parts(p as *const i16, len));
                }
                if self.cap.ReleaseBuffer(frames).is_err() {
                    break;
                }
            }
        }
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        // SAFETY: para o fluxo antes de soltar os objetos.
        let _ = unsafe { self.client.Stop() };
    }
}

fn enumerator() -> Res<IMMDeviceEnumerator> {
    // SAFETY: criação padrão do enumerador (COM já inicializado na thread).
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }.map_err(fail)
}
