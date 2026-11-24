//! Adaptador `Encoder`: Media Foundation (`IMFSinkWriter`), H.264 + AAC -> MP4.
//!
//! Sem FFmpeg: o MF já vem no Windows (e usa o encoder H.264 de hardware
//! quando existe). O arquivo só é criado no primeiro quadro de vídeo,
//! porque só então sabemos o tamanho real da captura (área/janela).

use super::com::{mf_init, wide};
use cap_core::domain::{Frame, RecCfg};
use cap_core::media::{
    audio_ts_ms, bgra_to_nv12, fit_bgra, has_audio, pack, video_bitrate, CH, RATE,
};
use cap_core::ports::Encoder;
use cap_core::{CapErr, Res};
use std::sync::Mutex;
use windows::core::{Error, GUID, PCWSTR};
use windows::Win32::Media::MediaFoundation::*;

/// Áudio AAC: 16 kB/s = 128 kbps (valor aceito pelo encoder AAC do MF).
const AAC_BYTES_PS: u32 = 16_000;
/// Unidades de 100 ns por milissegundo.
const HNS_MS: i64 = 10_000;
/// Limite de áudio guardado antes do 1º quadro de vídeo (~10 s).
const MAX_PEND: usize = 10 * RATE as usize * CH;

/// Encapsula um objeto COM do MF para poder ficar dentro de um `Mutex`
/// compartilhado entre threads (todo acesso é serializado pelo `Mutex`).
struct Sendable<T>(T);
// SAFETY: o sink writer do MF é livre de afinidade de thread (MTA) e o
// acesso é sempre protegido pelo Mutex do `MfEncoder`.
unsafe impl<T> Send for Sendable<T> {}

/// Escritor ativo (já iniciado).
struct Live {
    w: Sendable<IMFSinkWriter>,
    vs: u32,
    asx: Option<u32>,
    dw: u32,
    dh: u32,
}

/// Estado da gravação aberta.
struct Job {
    path: String,
    fps: u32,
    audio: bool,
    live: Option<Live>,
    /// Áudio que chegou antes do primeiro quadro: (pcm, ts_ms).
    pend: Vec<(Vec<i16>, u64)>,
    pend_len: usize,
    /// Primeira falha ao criar o arquivo (reportada no `finish`).
    err: Option<String>,
}

/// Encoder MP4 por Media Foundation.
pub struct MfEncoder {
    job: Mutex<Option<Job>>,
}

impl MfEncoder {
    pub fn new() -> Self {
        Self {
            job: Mutex::new(None),
        }
    }
}

/// Converte erro do Windows em mensagem para o usuário.
fn fail(what: &str, e: Error) -> CapErr {
    CapErr::from(format!("{what}: {}", e.message()))
}

/// Define um tipo de mídia com vários atributos de uma vez.
fn set_u32(t: &IMFMediaType, k: &GUID, v: u32) -> windows::core::Result<()> {
    // SAFETY: chamada COM com ponteiros válidos.
    unsafe { t.SetUINT32(k, v) }
}

/// Tipo de vídeo (H.264 de saída ou NV12 de entrada).
fn video_type(
    sub: GUID,
    w: u32,
    h: u32,
    fps: u32,
    bitrate: Option<u32>,
) -> windows::core::Result<IMFMediaType> {
    // SAFETY: APIs do MF com argumentos válidos.
    unsafe {
        let t = MFCreateMediaType()?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        t.SetGUID(&MF_MT_SUBTYPE, &sub)?;
        t.SetUINT64(&MF_MT_FRAME_SIZE, pack(w, h))?;
        t.SetUINT64(&MF_MT_FRAME_RATE, pack(fps, 1))?;
        t.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, pack(1, 1))?;
        set_u32(
            &t,
            &MF_MT_INTERLACE_MODE,
            MFVideoInterlace_Progressive.0 as u32,
        )?;
        // Cor: BT.709, faixa limitada (casa com `bgra_to_nv12`).
        set_u32(&t, &MF_MT_VIDEO_PRIMARIES, MFVideoPrimaries_BT709.0 as u32)?;
        set_u32(&t, &MF_MT_TRANSFER_FUNCTION, MFVideoTransFunc_709.0 as u32)?;
        set_u32(&t, &MF_MT_YUV_MATRIX, MFVideoTransferMatrix_BT709.0 as u32)?;
        set_u32(
            &t,
            &MF_MT_VIDEO_NOMINAL_RANGE,
            MFNominalRange_16_235.0 as u32,
        )?;
        if let Some(b) = bitrate {
            set_u32(&t, &MF_MT_AVG_BITRATE, b)?;
        }
        Ok(t)
    }
}

/// Tipo de áudio (AAC de saída ou PCM 16-bit de entrada), 48 kHz estéreo.
fn audio_type(sub: GUID, bytes_ps: u32) -> windows::core::Result<IMFMediaType> {
    // SAFETY: APIs do MF com argumentos válidos.
    unsafe {
        let t = MFCreateMediaType()?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
        t.SetGUID(&MF_MT_SUBTYPE, &sub)?;
        set_u32(&t, &MF_MT_AUDIO_BITS_PER_SAMPLE, 16)?;
        set_u32(&t, &MF_MT_AUDIO_SAMPLES_PER_SECOND, RATE)?;
        set_u32(&t, &MF_MT_AUDIO_NUM_CHANNELS, CH as u32)?;
        set_u32(&t, &MF_MT_AUDIO_AVG_BYTES_PER_SECOND, bytes_ps)?;
        if sub == MFAudioFormat_PCM {
            set_u32(&t, &MF_MT_AUDIO_BLOCK_ALIGNMENT, CH as u32 * 2)?;
        } else {
            set_u32(&t, &MF_MT_AUDIO_BLOCK_ALIGNMENT, 1)?;
            set_u32(&t, &MF_MT_AAC_PAYLOAD_TYPE, 0)?;
            // AAC-LC nível 2.
            set_u32(&t, &MF_MT_AAC_AUDIO_PROFILE_LEVEL_INDICATION, 0x29)?;
        }
        Ok(t)
    }
}

/// Cria o arquivo e inicia a escrita com o tamanho `w x h`.
fn begin(job: &Job, w: u32, h: u32) -> windows::core::Result<Live> {
    // SAFETY: APIs do MF com argumentos válidos; `path` vive durante a chamada.
    unsafe {
        let mut attrs = None;
        MFCreateAttributes(&mut attrs, 2)?;
        let attrs = attrs.ok_or_else(|| Error::from(windows::Win32::Foundation::E_FAIL))?;
        // Usa o encoder H.264 de hardware quando disponível.
        attrs.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1)?;
        attrs.SetUINT32(&MF_SINK_WRITER_DISABLE_THROTTLING, 1)?;
        let path = wide(&job.path);
        let wr = MFCreateSinkWriterFromURL(PCWSTR(path.as_ptr()), None, &attrs)?;
        let out = video_type(
            MFVideoFormat_H264,
            w,
            h,
            job.fps,
            Some(video_bitrate(w, h, job.fps)),
        )?;
        let vs = wr.AddStream(&out)?;
        let inp = video_type(MFVideoFormat_NV12, w, h, job.fps, None)?;
        set_u32(&inp, &MF_MT_DEFAULT_STRIDE, w)?;
        wr.SetInputMediaType(vs, &inp, None)?;
        let asx = if job.audio {
            let s = wr.AddStream(&audio_type(MFAudioFormat_AAC, AAC_BYTES_PS)?)?;
            let pcm = audio_type(MFAudioFormat_PCM, RATE * CH as u32 * 2)?;
            wr.SetInputMediaType(s, &pcm, None)?;
            Some(s)
        } else {
            None
        };
        wr.BeginWriting()?;
        Ok(Live {
            w: Sendable(wr),
            vs,
            asx,
            dw: w,
            dh: h,
        })
    }
}

/// Envia um bloco de bytes ao stream como amostra com tempo/duração (ms).
fn write(
    l: &Live,
    stream: u32,
    data: &[u8],
    ts_ms: u64,
    dur_hns: i64,
) -> windows::core::Result<()> {
    // SAFETY: o buffer é criado com o tamanho exato e só escrevemos dentro dele.
    unsafe {
        let buf = MFCreateMemoryBuffer(data.len() as u32)?;
        let mut p = std::ptr::null_mut();
        buf.Lock(&mut p, None, None)?;
        std::ptr::copy_nonoverlapping(data.as_ptr(), p, data.len());
        buf.Unlock()?;
        buf.SetCurrentLength(data.len() as u32)?;
        let s = MFCreateSample()?;
        s.AddBuffer(&buf)?;
        s.SetSampleTime(ts_ms as i64 * HNS_MS)?;
        s.SetSampleDuration(dur_hns)?;
        l.w.0.WriteSample(stream, &s)
    }
}

/// Escreve PCM (i16 estéreo) no stream de áudio.
fn write_pcm(l: &Live, pcm: &[i16], ts_ms: u64) -> windows::core::Result<()> {
    let Some(s) = l.asx else { return Ok(()) };
    // SAFETY: reinterpreta i16 como bytes (mesmo tamanho total, alinhamento menor).
    let bytes = unsafe { std::slice::from_raw_parts(pcm.as_ptr() as *const u8, pcm.len() * 2) };
    let frames = (pcm.len() / CH) as u64;
    let dur = (audio_ts_ms(frames * 1000) as i64) * HNS_MS / 1000;
    write(l, s, bytes, ts_ms, dur)
}

impl Encoder for MfEncoder {
    fn open(&self, path: &str, cfg: &RecCfg) -> Res<()> {
        mf_init();
        *crate::lk(&self.job) = Some(Job {
            path: path.to_string(),
            fps: cfg.fps.max(1),
            audio: has_audio(cfg),
            live: None,
            pend: Vec::new(),
            pend_len: 0,
            err: None,
        });
        Ok(())
    }

    fn write_video(&self, f: &Frame) -> Res<()> {
        let mut g = crate::lk(&self.job);
        let job = g.as_mut().ok_or("Nenhuma gravação aberta.")?;
        if job.err.is_some() {
            return Ok(());
        }
        // O primeiro quadro define o tamanho do vídeo (sempre par).
        if job.live.is_none() {
            let (w, h) = (f.w & !1, f.h & !1);
            match begin(job, w, h) {
                Ok(l) => {
                    // Descarrega o áudio que chegou antes.
                    for (pcm, ts) in std::mem::take(&mut job.pend) {
                        let _ = write_pcm(&l, &pcm, ts);
                    }
                    job.live = Some(l);
                }
                Err(e) => {
                    let m = fail("Falha ao criar o arquivo MP4", e).0;
                    job.err = Some(m.clone());
                    return Err(CapErr(m));
                }
            }
        }
        let fps = job.fps;
        let l = job.live.as_ref().ok_or("Nenhuma gravação aberta.")?;
        // Quadros de outro tamanho (janela redimensionada) são encaixados.
        let nv = if (f.w, f.h) == (l.dw, l.dh) {
            bgra_to_nv12(&f.data, l.dw, l.dh)
        } else {
            bgra_to_nv12(&fit_bgra(&f.data, f.w, f.h, l.dw, l.dh), l.dw, l.dh)
        };
        write(l, l.vs, &nv, f.ts_ms, 10_000_000 / fps as i64)
            .map_err(|e| fail("Falha ao gravar o vídeo", e))
    }

    fn write_audio(&self, pcm: &[i16], ts_ms: u64) -> Res<()> {
        let mut g = crate::lk(&self.job);
        let job = g.as_mut().ok_or("Nenhuma gravação aberta.")?;
        if !job.audio {
            return Ok(());
        }
        match &job.live {
            Some(l) => write_pcm(l, pcm, ts_ms).map_err(|e| fail("Falha ao gravar o áudio", e)),
            None => {
                // Ainda sem vídeo: guarda (com limite) para descarregar depois.
                if job.pend_len + pcm.len() <= MAX_PEND {
                    job.pend_len += pcm.len();
                    job.pend.push((pcm.to_vec(), ts_ms));
                }
                Ok(())
            }
        }
    }

    fn finish(&self) -> Res<()> {
        let job = crate::lk(&self.job)
            .take()
            .ok_or("Nenhuma gravação aberta.")?;
        if let Some(m) = job.err {
            return Err(CapErr(m));
        }
        let l = job
            .live
            .ok_or("Nenhum quadro foi gravado; o arquivo não foi criado.")?;
        // SAFETY: Finalize grava o índice do MP4; o writer é solto no fim.
        unsafe { l.w.0.Finalize() }.map_err(|e| fail("Falha ao finalizar o MP4", e))
    }
}
