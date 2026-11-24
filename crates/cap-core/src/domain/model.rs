//! Tipos do contrato (JSON camelCase).

use serde::{Deserialize, Serialize};

/// Região da tela em pixels físicos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

/// Modo de captura.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Screen,
    Window,
    Area,
}

/// Monitor disponível para captura.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Monitor {
    pub id: u32,
    pub name: String,
    pub w: u32,
    pub h: u32,
    pub x: i32,
    pub y: i32,
    pub primary: bool,
}

/// Janela disponível para captura.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Win {
    pub id: u32,
    pub title: String,
}

/// Dispositivo de entrada (câmera ou microfone).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dev {
    pub id: String,
    pub name: String,
}

/// Alvos de captura: monitores e janelas.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Targets {
    pub monitors: Vec<Monitor>,
    pub wins: Vec<Win>,
}

/// Dispositivos: câmeras e microfones.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Devices {
    pub cams: Vec<Dev>,
    pub mics: Vec<Dev>,
}

/// Configuração de uma gravação.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecCfg {
    pub mode: Mode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<Region>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mic_id: Option<String>,
    pub sys: bool,
    pub fps: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
}

/// Fase da gravação.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum St {
    Idle,
    Rec,
    Paused,
}

/// Estado da gravação com cronômetro em segundos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecState {
    pub state: St,
    pub secs: u64,
}

impl RecState {
    /// Estado ocioso, cronômetro zerado.
    pub fn idle() -> Self {
        RecState {
            state: St::Idle,
            secs: 0,
        }
    }
}

/// Atualização encontrada pelo `Updater`: só a versão (sem 'v').
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdInfo {
    pub version: String,
}

/// Estado do ciclo de atualização, emitido como `update-state`.
///
/// `Fallback` quer dizer que o app não foi instalado pelo instalador com
/// atualização automática: a interface cai no aviso por consulta ao GitHub.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum UpdState {
    Idle,
    Checking,
    None,
    Downloading { pct: u8 },
    Ready { version: String },
    Fallback,
    Err,
}

/// Quadro de vídeo bruto (BGRA) entregue pela captura ao encoder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub w: u32,
    pub h: u32,
    pub ts_ms: u64,
    pub data: Vec<u8>,
}

/// Data/hora local simples, fornecida pelo relógio injetado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    pub y: u16,
    pub mo: u8,
    pub d: u8,
    pub h: u8,
    pub mi: u8,
    pub s: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn region_json() {
        let r = Region {
            x: -5,
            y: 2,
            w: 30,
            h: 40,
        };
        assert_eq!(
            serde_json::to_value(r).unwrap(),
            json!({"x":-5,"y":2,"w":30,"h":40})
        );
    }

    #[test]
    fn cfg_camel_case_completa() {
        let v = json!({"mode":"area","targetId":3,"region":{"x":0,"y":0,"w":50,"h":50},
            "micId":"m1","sys":true,"fps":30,"dir":"C:\\v"});
        let c: RecCfg = serde_json::from_value(v.clone()).unwrap();
        assert_eq!(c.mode, Mode::Area);
        assert_eq!(c.target_id, Some(3));
        assert_eq!(c.mic_id.as_deref(), Some("m1"));
        assert_eq!(serde_json::to_value(&c).unwrap(), v);
    }

    #[test]
    fn cfg_minima_omite_opcionais() {
        let c: RecCfg =
            serde_json::from_value(json!({"mode":"screen","sys":false,"fps":24})).unwrap();
        assert_eq!(c.region, None);
        assert_eq!(
            serde_json::to_value(&c).unwrap(),
            json!({"mode":"screen","sys":false,"fps":24})
        );
    }

    #[test]
    fn modos_e_estados() {
        assert_eq!(serde_json::to_string(&Mode::Window).unwrap(), "\"window\"");
        assert_eq!(serde_json::to_string(&Mode::Screen).unwrap(), "\"screen\"");
        let s = RecState {
            state: St::Paused,
            secs: 7,
        };
        assert_eq!(
            serde_json::to_value(s).unwrap(),
            json!({"state":"paused","secs":7})
        );
        assert_eq!(
            serde_json::to_value(RecState::idle()).unwrap(),
            json!({"state":"idle","secs":0})
        );
        assert_eq!(serde_json::to_string(&St::Rec).unwrap(), "\"rec\"");
    }

    #[test]
    fn estados_de_atualizacao() {
        let j = |s: UpdState| serde_json::to_value(s).unwrap();
        assert_eq!(j(UpdState::Checking), json!({"kind":"checking"}));
        assert_eq!(
            j(UpdState::Downloading { pct: 42 }),
            json!({"kind":"downloading","pct":42})
        );
        assert_eq!(
            j(UpdState::Ready {
                version: "1.2.3".into()
            }),
            json!({"kind":"ready","version":"1.2.3"})
        );
        assert_eq!(j(UpdState::Fallback), json!({"kind":"fallback"}));
        let back: UpdState = serde_json::from_value(json!({"kind":"none"})).unwrap();
        assert_eq!(back, UpdState::None);
        assert_eq!(
            UpdInfo {
                version: "1".into()
            }
            .clone()
            .version,
            "1"
        );
    }

    #[test]
    fn targets_e_devices() {
        let t = Targets {
            monitors: vec![Monitor {
                id: 1,
                name: "M".into(),
                w: 10,
                h: 20,
                x: 0,
                y: 0,
                primary: true,
            }],
            wins: vec![Win {
                id: 9,
                title: "J".into(),
            }],
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["monitors"][0]["primary"], json!(true));
        assert_eq!(v["wins"][0]["title"], json!("J"));
        let d = Devices {
            cams: vec![Dev {
                id: "c".into(),
                name: "Cam".into(),
            }],
            mics: vec![],
        };
        assert_eq!(
            serde_json::to_value(&d).unwrap(),
            json!({"cams":[{"id":"c","name":"Cam"}],"mics":[]})
        );
        assert_eq!(Targets::default().wins.len(), 0);
        assert_eq!(Devices::default().mics.len(), 0);
    }

    #[test]
    fn frame_e_stamp() {
        let f = Frame {
            w: 1,
            h: 1,
            ts_ms: 0,
            data: vec![0; 4],
        };
        assert_eq!(f.clone(), f);
        let s = Stamp {
            y: 2026,
            mo: 1,
            d: 2,
            h: 3,
            mi: 4,
            s: 5,
        };
        assert_eq!(s, s);
    }
}
