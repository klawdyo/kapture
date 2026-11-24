//! Captura de áudio com WASAPI (modo compartilhado, por polling):
//! microfone e loopback do sistema.
//!
//! O fluxo é aberto SEMPRE no formato nativo do mix do dispositivo
//! (`GetMixFormat`): `AUTOCONVERTPCM` não é aceito junto de `LOOPBACK` e
//! `SRC_DEFAULT_QUALITY` só vale com `AUTOCONVERTPCM`; qualquer um deles
//! devolvia `E_INVALIDARG`. A conversão para i16 estéreo 48 kHz é feita em
//! software por `cap_core::media::Conv` (lógica pura, testada).

use super::super::com::{log_err, take_pwstr, wide};
use crate::logging;
use cap_core::log::hresult;
use cap_core::media::{audio_fail_line, fmt_desc, parse_fmt, Conv};
use cap_core::{CapErr, Res};
use windows::core::PCWSTR;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::{
    eConsole, eRender, IAudioCaptureClient, IAudioClient, IMMDevice, IMMDeviceEnumerator,
    MMDeviceEnumerator, AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED,
    AUDCLNT_STREAMFLAGS_LOOPBACK, WAVEFORMATEX, WAVEFORMATEXTENSIBLE,
};
use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PropVariantToStringAlloc};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL, STGM_READ};

/// Leitor de uma fonte de áudio (já convertido para i16 estéreo 48 kHz).
pub struct Reader {
    client: IAudioClient,
    cap: IAudioCaptureClient,
    conv: Conv,
}

/// Registra uma falha de uma etapa da abertura (com dispositivo e formato) e devolve o erro.
fn fail(step: &str, dev: &str, fmt: &str, e: windows::core::Error) -> CapErr {
    logging::get().error(&audio_fail_line(step, dev, fmt, e.code().0, &e.message()));
    CapErr::from(format!(
        "Falha ao abrir o áudio ({step}, {})",
        hresult(e.code().0)
    ))
}

/// Nome amigável do dispositivo (ou o id, ou "?" se nada funcionar).
fn dev_name(dev: &IMMDevice) -> String {
    // SAFETY: APIs COM padrão; strings alocadas são liberadas em `take_pwstr`.
    unsafe {
        let id = dev.GetId().map(take_pwstr).unwrap_or_else(|_| "?".into());
        let Ok(store) = dev.OpenPropertyStore(STGM_READ) else {
            return id;
        };
        let Ok(mut pv) = store.GetValue(&PKEY_Device_FriendlyName) else {
            return id;
        };
        let name = PropVariantToStringAlloc(&pv).map(take_pwstr).unwrap_or(id);
        let _ = PropVariantClear(&mut pv);
        name
    }
}

/// Lê o formato do mix (copiando os campos) e libera o bloco do COM.
fn read_mix(client: &IAudioClient) -> windows::core::Result<(WAVEFORMATEX, u16)> {
    // SAFETY: `GetMixFormat` devolve um bloco do COM com ao menos um
    // WAVEFORMATEX; com a etiqueta extensível ele é um WAVEFORMATEXTENSIBLE
    // (lido sem alinhamento, pois a estrutura é `packed`). Liberado uma vez.
    unsafe {
        let p = client.GetMixFormat()?;
        let f = p.read_unaligned();
        let mut sub = 0u16;
        if f.wFormatTag == cap_core::media::TAG_EXT {
            let x = p as *const WAVEFORMATEXTENSIBLE;
            sub = std::ptr::addr_of!((*x).SubFormat).read_unaligned().data1 as u16;
        }
        CoTaskMemFree(Some(p as *const _));
        Ok((f, sub))
    }
}

impl Reader {
    /// Microfone pelo id do endpoint.
    pub fn mic(id: &str) -> Res<Reader> {
        let en = enumerator()?;
        let w = wide(id);
        // SAFETY: `w` é terminado em nulo e vive durante a chamada.
        let dev = unsafe { en.GetDevice(PCWSTR(w.as_ptr())) }.map_err(|e| {
            log_err(&format!("microfone: GetDevice({id})"), &e);
            CapErr::from("O microfone escolhido não está disponível.")
        })?;
        Reader::open(&dev, false)
    }

    /// Som do sistema (loopback do dispositivo de saída padrão).
    pub fn system() -> Res<Reader> {
        let en = enumerator()?;
        // SAFETY: consulta simples ao enumerador.
        let dev = unsafe { en.GetDefaultAudioEndpoint(eRender, eConsole) }.map_err(|e| {
            log_err("som do sistema: GetDefaultAudioEndpoint", &e);
            CapErr::from("Nenhuma saída de áudio padrão foi encontrada.")
        })?;
        Reader::open(&dev, true)
    }

    fn open(dev: &IMMDevice, loopback: bool) -> Res<Reader> {
        let name = dev_name(dev);
        let kind = if loopback {
            "som do sistema"
        } else {
            "microfone"
        };
        // SAFETY: sequência padrão de inicialização do WASAPI; o formato do
        // mix é lido por `read_mix` e só é usado dentro de `Initialize`.
        unsafe {
            let client: IAudioClient = dev
                .Activate(CLSCTX_ALL, None)
                .map_err(|e| fail("Activate", &name, "?", e))?;
            let (mix, sub) = read_mix(&client).map_err(|e| fail("GetMixFormat", &name, "?", e))?;
            let (tag, bits, ch, rate) = (
                mix.wFormatTag,
                mix.wBitsPerSample,
                mix.nChannels,
                mix.nSamplesPerSec,
            );
            let desc = format!("tag {tag:#06x}, {bits} bits, {ch} canais, {rate} Hz");
            let Some(fmt) = parse_fmt(tag, bits, ch, rate, sub) else {
                logging::get().error(&format!(
                    "áudio: formato do mix não suportado (dispositivo '{name}', {kind}): {desc}"
                ));
                return Err(CapErr::from(
                    "Formato de áudio do dispositivo não suportado.",
                ));
            };
            let fd = fmt_desc(&fmt);
            // Só o loopback leva flag; o formato é o nativo (sem conversão do WASAPI).
            let flags = if loopback {
                AUDCLNT_STREAMFLAGS_LOOPBACK
            } else {
                0
            };
            // Buffer de 200 ms (unidades de 100 ns).
            client
                .Initialize(AUDCLNT_SHAREMODE_SHARED, flags, 2_000_000, 0, &mix, None)
                .map_err(|e| fail("Initialize", &name, &fd, e))?;
            let cap: IAudioCaptureClient = client
                .GetService()
                .map_err(|e| fail("GetService", &name, &fd, e))?;
            client.Start().map_err(|e| fail("Start", &name, &fd, e))?;
            logging::get().info(&format!(
                "áudio: leitor aberto ({kind}, '{name}', {fd}, flags {flags:#x})"
            ));
            Ok(Reader {
                client,
                cap,
                conv: Conv::new(fmt),
            })
        }
    }

    /// Acrescenta a `out` todas as amostras (i16 estéreo 48 kHz) disponíveis agora.
    pub fn read(&mut self, out: &mut Vec<i16>) {
        // SAFETY: cada GetBuffer é pareado com ReleaseBuffer; o slice cobre
        // exatamente `frames * bytes por quadro` bytes do buffer devolvido.
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
                if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 || p.is_null() {
                    self.conv.push_silence(frames as usize, out);
                } else {
                    let len = frames as usize * self.conv.block();
                    self.conv.push(std::slice::from_raw_parts(p, len), out);
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
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
        .map_err(|e| fail("CoCreateInstance(MMDeviceEnumerator)", "?", "?", e))
}
