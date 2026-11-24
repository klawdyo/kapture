//! Captura de vídeo com Windows Graphics Capture (WGC), escrita direto
//! sobre a API `windows` (sem a crate `windows-capture`): ela traz
//! rayon, parking_lot e dezenas de features WinRT que não usamos, o que
//! aumentaria o binário; aqui são ~150 linhas.
//!
//! Cada quadro é copiado da GPU para uma textura de "staging", recortado
//! (área) e guardado como o "último quadro"; a thread de captura o envia
//! ao encoder no ritmo constante de FPS.

use super::super::com::log_err;
use crate::lk;
use crate::logging;
use cap_core::domain::Region;
use cap_core::media::{crop_bgra, Source};
use cap_core::{CapErr, Res};
use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use windows::core::{factory, Interface};
use windows::Foundation::TypedEventHandler;
use windows::Graphics::Capture::{
    Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession,
};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Win32::Foundation::{E_POINTER, HMODULE, HWND};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, D3D11_CPU_ACCESS_READ,
    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ, D3D11_SDK_VERSION,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC;
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::Graphics::Gdi::HMONITOR;
use windows::Win32::System::WinRT::Direct3D11::{
    CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;

/// Quadro BGRA compacto, já recortado.
#[derive(Clone)]
pub struct Img {
    pub w: u32,
    pub h: u32,
    pub data: Vec<u8>,
}

/// Último quadro recebido.
pub type Latest = Arc<Mutex<Option<Img>>>;

/// Recursos de GPU usados dentro do callback de quadros.
struct Gpu {
    dev: ID3D11Device,
    ctx: ID3D11DeviceContext,
    /// Textura de staging e suas dimensões.
    stg: Option<(ID3D11Texture2D, u32, u32)>,
}

/// Captura ativa; soltar encerra tudo.
pub struct Grab {
    pub latest: Latest,
    pool: Direct3D11CaptureFramePool,
    session: GraphicsCaptureSession,
    token: i64,
}

/// Converte erro do Windows numa mensagem em português.
fn err(what: &str, e: windows::core::Error) -> CapErr {
    log_err(what, &e);
    CapErr::from(format!("{what}: {}", e.message()))
}

impl Grab {
    /// Inicia a captura da origem, com recorte opcional (coordenadas locais).
    pub fn new(src: Source, crop: Option<Region>) -> Res<Grab> {
        let ok = GraphicsCaptureSession::IsSupported().unwrap_or(false);
        if !ok {
            logging::get().error("captura de vídeo: Windows Graphics Capture não é suportado");
            return Err("A captura de tela exige o Windows 10 versão 1903 ou superior.".into());
        }
        let (dev, ctx) = d3d_device()?;
        let dxgi: IDXGIDevice = dev
            .cast()
            .map_err(|e| err("Falha ao preparar a placa de vídeo", e))?;
        // SAFETY: cria o dispositivo WinRT a partir do DXGI válido.
        let winrt: IDirect3DDevice = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi) }
            .and_then(|i| i.cast())
            .map_err(|e| err("Falha ao preparar a placa de vídeo", e))?;
        let item = item_for(src)?;
        let size = item
            .Size()
            .map_err(|e| err("Falha ao ler o tamanho da captura", e))?;
        let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &winrt,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            2,
            size,
        )
        .map_err(|e| err("Falha ao iniciar a captura", e))?;

        let latest: Latest = Arc::new(Mutex::new(None));
        let sink = latest.clone();
        let gpu = Mutex::new(Gpu {
            dev,
            ctx,
            stg: None,
        });
        let token = pool
            .FrameArrived(&TypedEventHandler::new(
                move |p: windows::core::Ref<Direct3D11CaptureFramePool>, _| {
                    let r = (|| -> windows::core::Result<()> {
                        let p = p.ok()?;
                        let frame = p.TryGetNextFrame()?;
                        let sz = frame.ContentSize()?;
                        let access: IDirect3DDxgiInterfaceAccess = frame.Surface()?.cast()?;
                        // SAFETY: a superfície do WGC é uma ID3D11Texture2D.
                        let tex: ID3D11Texture2D = unsafe { access.GetInterface()? };
                        let img = read_frame(
                            &mut lk(&gpu),
                            &tex,
                            sz.Width as u32,
                            sz.Height as u32,
                            crop,
                        )?;
                        *lk(&sink) = img;
                        Ok(())
                    })();
                    if let Err(e) = &r {
                        frame_err(e);
                    }
                    r
                },
            ))
            .map_err(|e| err("Falha ao iniciar a captura", e))?;
        let session = pool
            .CreateCaptureSession(&item)
            .map_err(|e| err("Falha ao iniciar a captura", e))?;
        // O cursor aparece no vídeo.
        let _ = session.SetIsCursorCaptureEnabled(true);
        session
            .StartCapture()
            .map_err(|e| err("Falha ao iniciar a captura", e))?;
        logging::get().info(&format!(
            "captura de vídeo: iniciada (origem {src:?}, {}x{}, recorte {crop:?})",
            size.Width, size.Height
        ));
        Ok(Grab {
            latest,
            pool,
            session,
            token,
        })
    }
}

/// Registra falhas ao ler quadros (a 1ª e depois uma a cada 300, para
/// não encher o arquivo a 30 quadros por segundo).
fn frame_err(e: &windows::core::Error) {
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed) + 1;
    if n == 1 || n.is_multiple_of(300) {
        log_err(&format!("captura de vídeo: falha ao ler quadro (#{n})"), e);
    }
}

impl Drop for Grab {
    fn drop(&mut self) {
        logging::get().info("captura de vídeo: encerrada");
        // Ordem: para a sessão, desliga o callback e fecha o pool.
        let _ = self.session.Close();
        let _ = self.pool.RemoveFrameArrived(self.token);
        let _ = self.pool.Close();
    }
}

/// Cria o dispositivo D3D11 (GPU) usado só para copiar quadros.
fn d3d_device() -> Res<(ID3D11Device, ID3D11DeviceContext)> {
    let (mut dev, mut ctx) = (None, None);
    // SAFETY: ponteiros de saída válidos.
    unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut dev),
            None,
            Some(&mut ctx),
        )
    }
    .map_err(|e| err("Falha ao acessar a placa de vídeo", e))?;
    match (dev, ctx) {
        (Some(d), Some(c)) => Ok((d, c)),
        _ => Err("Falha ao acessar a placa de vídeo.".into()),
    }
}

/// Item de captura (monitor ou janela) a partir do id do contrato.
fn item_for(src: Source) -> Res<GraphicsCaptureItem> {
    let interop = factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
        .map_err(|e| err("Falha ao iniciar a captura", e))?;
    // SAFETY: os ids são HMONITOR/HWND listados por `targets`.
    match src {
        Source::Monitor(id) => {
            unsafe { interop.CreateForMonitor(HMONITOR(id as usize as *mut c_void)) }
                .map_err(|e| err("Não foi possível capturar o monitor escolhido", e))
        }
        Source::Window(id) => unsafe { interop.CreateForWindow(HWND(id as usize as *mut c_void)) }
            .map_err(|e| {
                err(
                    "Não foi possível capturar a janela escolhida (ela ainda está aberta?)",
                    e,
                )
            }),
    }
}

/// Copia a textura para a CPU e recorta; `None` se o recorte ficou vazio.
fn read_frame(
    g: &mut Gpu,
    tex: &ID3D11Texture2D,
    cw: u32,
    ch: u32,
    crop: Option<Region>,
) -> windows::core::Result<Option<Img>> {
    let mut d = D3D11_TEXTURE2D_DESC::default();
    // SAFETY: `d` é válido.
    unsafe { tex.GetDesc(&mut d) };
    // Recria o staging se o tamanho da textura mudou.
    if g.stg.as_ref().map(|s| (s.1, s.2)) != Some((d.Width, d.Height)) {
        let sd = D3D11_TEXTURE2D_DESC {
            Width: d.Width,
            Height: d.Height,
            MipLevels: 1,
            ArraySize: 1,
            Format: d.Format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
        };
        let mut t = None;
        // SAFETY: descrição válida; `t` recebe a textura criada.
        unsafe { g.dev.CreateTexture2D(&sd, None, Some(&mut t))? };
        let t = t.ok_or_else(|| windows::core::Error::from(E_POINTER))?;
        g.stg = Some((t, d.Width, d.Height));
    }
    let stg = &g
        .stg
        .as_ref()
        .ok_or_else(|| windows::core::Error::from(E_POINTER))?
        .0;
    let mut m = D3D11_MAPPED_SUBRESOURCE::default();
    // SAFETY: copia GPU->staging, mapeia para leitura e desmapeia no fim;
    // o slice cobre exatamente `RowPitch * Height` bytes mapeados.
    let img = unsafe {
        g.ctx.CopyResource(stg, tex);
        g.ctx.Map(stg, 0, D3D11_MAP_READ, 0, Some(&mut m))?;
        let data = std::slice::from_raw_parts(
            m.pData as *const u8,
            m.RowPitch as usize * d.Height as usize,
        );
        let (w, h) = (cw.min(d.Width), ch.min(d.Height));
        let r = crop.unwrap_or(Region { x: 0, y: 0, w, h });
        let out = crop_bgra(data, m.RowPitch as usize, w, h, &r);
        g.ctx.Unmap(stg, 0);
        out
    };
    Ok(img.map(|(w, h, data)| Img { w, h, data }))
}
