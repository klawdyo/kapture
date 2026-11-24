//! Adaptador `Devs`: câmeras (Media Foundation) e microfones (MMDevice).
//!
//! Atenção: o `id` das câmeras é o NOME amigável. O WebView2 gera
//! deviceIds próprios (hash) no `getUserMedia`, que não existem no
//! Windows; a janela da webcam deve casar a câmera pelo `label`.

use super::com::{log_err, mf_init, take_pwstr};
use cap_core::domain::{Dev, Devices};
use cap_core::ports::Devs;
use cap_core::{CapErr, Res};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::{
    eCapture, IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
};
use windows::Win32::Media::MediaFoundation::{
    IMFActivate, MFCreateAttributes, MFEnumDeviceSources, MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
    MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
};
use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PropVariantToStringAlloc};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL, STGM_READ};

/// Enumerador real de dispositivos.
pub struct WinDevs;

/// Câmeras de vídeo.
fn cams() -> windows::core::Result<Vec<Dev>> {
    let mut out = Vec::new();
    // SAFETY: APIs do Media Foundation; o array devolvido é liberado com
    // CoTaskMemFree e cada IMFActivate é solto pelo `Drop`.
    unsafe {
        let mut attrs = None;
        MFCreateAttributes(&mut attrs, 1)?;
        let attrs = attrs.ok_or(windows::core::Error::from_hresult(
            windows::Win32::Foundation::E_FAIL,
        ))?;
        attrs.SetGUID(
            &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
            &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
        )?;
        let mut arr: *mut Option<IMFActivate> = std::ptr::null_mut();
        let mut n = 0u32;
        MFEnumDeviceSources(&attrs, &mut arr, &mut n)?;
        for i in 0..n as usize {
            if let Some(a) = (*arr.add(i)).take() {
                let mut p = windows::core::PWSTR::null();
                let mut len = 0u32;
                if a.GetAllocatedString(&MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME, &mut p, &mut len)
                    .is_ok()
                {
                    let name = take_pwstr(p);
                    out.push(Dev {
                        id: name.clone(),
                        name,
                    });
                }
            }
        }
        CoTaskMemFree(Some(arr as *const _));
    }
    Ok(out)
}

/// Microfones ativos (id = ID do endpoint MMDevice).
fn mics() -> windows::core::Result<Vec<Dev>> {
    let mut out = Vec::new();
    // SAFETY: APIs COM padrão; strings alocadas são liberadas em `take_pwstr`.
    unsafe {
        let en: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let col = en.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)?;
        for i in 0..col.GetCount()? {
            let d = col.Item(i)?;
            let id = take_pwstr(d.GetId()?);
            let store = d.OpenPropertyStore(STGM_READ)?;
            let mut pv = store.GetValue(&PKEY_Device_FriendlyName)?;
            let name = PropVariantToStringAlloc(&pv)
                .map(take_pwstr)
                .unwrap_or(id.clone());
            let _ = PropVariantClear(&mut pv);
            out.push(Dev { id, name });
        }
    }
    Ok(out)
}

impl Devs for WinDevs {
    fn list(&self) -> Res<Devices> {
        mf_init();
        // Falha ao listar câmeras não impede listar microfones (e vice-versa).
        let cams = cams().unwrap_or_else(|e| {
            log_err("listar câmeras (MFEnumDeviceSources)", &e);
            Vec::new()
        });
        let mics = mics().map_err(|e| {
            log_err("listar microfones (MMDevice)", &e);
            CapErr::from("Não foi possível listar os microfones.")
        })?;
        Ok(Devices { cams, mics })
    }
}
