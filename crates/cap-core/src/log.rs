//! Log em disco: lógica pura (formato da linha, rotação por tamanho e
//! truncamento). A escrita no arquivo fica em `logfile`; aqui não há E/S.

use crate::domain::RecCfg;
use crate::media::join_win;
use crate::ports::Log;
use std::any::Any;
use std::fmt::Display;

/// Tamanho máximo do arquivo ativo antes de rotacionar (512 KB).
pub const MAX_BYTES: u64 = 512 * 1024;
/// Quantidade máxima de arquivos (`kapture.log`, `kapture.1.log`, `kapture.2.log`).
pub const KEEP: usize = 3;
/// Tamanho máximo de uma mensagem, em bytes.
pub const MAX_MSG: usize = 4000;

/// Nível de severidade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lvl {
    Debug,
    Info,
    Warn,
    Error,
}

impl Lvl {
    /// Rótulo de largura fixa (alinha as colunas do arquivo).
    pub fn tag(self) -> &'static str {
        match self {
            Lvl::Debug => "DEBUG",
            Lvl::Info => "INFO ",
            Lvl::Warn => "WARN ",
            Lvl::Error => "ERROR",
        }
    }

    /// Nível a partir de texto (vindo da interface); desconhecido vira `Info`.
    pub fn parse(s: &str) -> Lvl {
        match s.to_ascii_lowercase().as_str() {
            "debug" => Lvl::Debug,
            "warn" | "warning" => Lvl::Warn,
            "error" => Lvl::Error,
            _ => Lvl::Info,
        }
    }
}

/// Converte dias desde 1970-01-01 em (ano, mês, dia) do calendário gregoriano.
fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// Milissegundos desde a época Unix como `2026-01-02T03:04:05.678Z` (UTC).
pub fn ts_iso(ms: u64) -> String {
    let secs = ms / 1000;
    let (y, m, d) = civil((secs / 86_400) as i64);
    let rem = secs % 86_400;
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{:03}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60,
        ms % 1000
    )
}

/// Corta `s` em no máximo `max` bytes (sem partir um caractere) e avisa
/// quantos bytes sobraram.
pub fn clip(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut n = max;
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    format!("{}... [+{} bytes truncados]", &s[..n], s.len() - n)
}

/// Linha pronta para o arquivo: `<ts> <NÍVEL> <mensagem>\n`.
/// Quebras de linha da mensagem viram continuação indentada.
pub fn line(ms: u64, lvl: Lvl, msg: &str) -> String {
    let body = clip(msg, MAX_MSG).replace('\r', "").replace('\n', "\n    ");
    format!("{} {} {}\n", ts_iso(ms), lvl.tag(), body)
}

/// Nome do arquivo de índice `i` (0 = ativo).
pub fn file_name(i: usize) -> String {
    if i == 0 {
        "kapture.log".to_string()
    } else {
        format!("kapture.{i}.log")
    }
}

/// Passo da rotação de arquivos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Remove(String),
    Rename(String, String),
}

/// Passos para abrir espaço para um novo `kapture.log` mantendo `keep` arquivos:
/// apaga o mais antigo e desloca os demais (`kapture.log` -> `kapture.1.log` ...).
pub fn rotation(keep: usize) -> Vec<Step> {
    let keep = keep.max(1);
    let mut v = vec![Step::Remove(file_name(keep - 1))];
    for i in (0..keep - 1).rev() {
        v.push(Step::Rename(file_name(i), file_name(i + 1)));
    }
    v
}

/// Deve rotacionar antes de gravar `add` bytes num arquivo com `size`?
/// Arquivo vazio nunca rotaciona (uma linha enorme não pode girar em falso).
pub fn needs_rotation(size: u64, add: u64, max: u64) -> bool {
    size > 0 && size + add > max
}

/// HRESULT em hexadecimal (`0x80070005`).
pub fn hresult(code: i32) -> String {
    format!("0x{:08X}", code as u32)
}

/// Pasta de logs: `<LocalAppData>\kapture\logs`, ou `<temp>\kapture\logs` sem a base.
pub fn logs_dir(base: Option<&str>, tmp: &str) -> String {
    let root = base.filter(|b| !b.is_empty()).unwrap_or(tmp);
    join_win(&join_win(root, "kapture"), "logs")
}

/// Linha de abertura do app.
pub fn start_line(ver: &str, os: &str, args: &[String]) -> String {
    format!("kapture {ver} iniciado; Windows {os}; args: {args:?}")
}

/// Resumo da configuração de gravação (o id do microfone não é registrado).
pub fn cfg_line(c: &RecCfg) -> String {
    format!(
        "modo={:?} alvo={:?} regiao={:?} mic={} sys={} fps={} dir={:?}",
        c.mode,
        c.target_id,
        c.region,
        if c.mic_id.as_deref().is_some_and(|m| !m.is_empty()) {
            "sim"
        } else {
            "nao"
        },
        c.sys,
        c.fps,
        c.dir
    )
}

/// Texto do pânico: mensagem, local (`arquivo:linha`) e backtrace.
pub fn panic_text(payload: &str, loc: Option<(&str, u32)>, thread: &str, bt: &str) -> String {
    let at = match loc {
        Some((f, l)) => format!("{f}:{l}"),
        None => "local desconhecido".to_string(),
    };
    format!("PANICO na thread '{thread}' em {at}: {payload}\n{bt}")
}

/// Extrai a mensagem do payload de um pânico (`&str` ou `String`).
pub fn payload_str(p: &(dyn Any + Send)) -> String {
    if let Some(s) = p.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = p.downcast_ref::<String>() {
        s.clone()
    } else {
        "(sem mensagem)".to_string()
    }
}

/// Registra o resultado de um comando IPC (nível `Error` se falhou).
pub fn log_ipc<T, E: Display>(log: &dyn Log, name: &str, ms: u64, r: &Result<T, E>) {
    match r {
        Ok(_) => log.info(&format!("ipc {name}: ok ({ms} ms)")),
        Err(e) => log.error(&format!("ipc {name}: erro: {e} ({ms} ms)")),
    }
}

/// Registra uma mensagem vinda da interface (comando `log_ui`).
pub fn log_ui(log: &dyn Log, lvl: &str, msg: &str) {
    log.write(Lvl::parse(lvl), &format!("[ui] {msg}"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Mode, Region};
    use crate::fakes::FakeLog;

    #[test]
    fn niveis() {
        assert_eq!(Lvl::Debug.tag(), "DEBUG");
        assert_eq!(Lvl::Info.tag(), "INFO ");
        assert_eq!(Lvl::Warn.tag(), "WARN ");
        assert_eq!(Lvl::Error.tag(), "ERROR");
        assert_eq!(Lvl::parse("DEBUG"), Lvl::Debug);
        assert_eq!(Lvl::parse("warn"), Lvl::Warn);
        assert_eq!(Lvl::parse("Warning"), Lvl::Warn);
        assert_eq!(Lvl::parse("error"), Lvl::Error);
        assert_eq!(Lvl::parse("info"), Lvl::Info);
        assert_eq!(Lvl::parse("qualquer"), Lvl::Info);
    }

    #[test]
    fn data_iso() {
        assert_eq!(ts_iso(0), "1970-01-01T00:00:00.000Z");
        // 2026-01-02T03:04:05.678Z
        assert_eq!(ts_iso(1_767_323_045_678), "2026-01-02T03:04:05.678Z");
        // Ano bissexto: 2024-02-29 e virada de ano.
        assert_eq!(ts_iso(1_709_164_800_000), "2024-02-29T00:00:00.000Z");
        assert_eq!(ts_iso(1_767_225_599_999), "2025-12-31T23:59:59.999Z");
        assert_eq!(ts_iso(951_782_400_000), "2000-02-29T00:00:00.000Z");
    }

    #[test]
    fn corta_mensagens() {
        assert_eq!(clip("abc", 3), "abc");
        assert_eq!(clip("abcdef", 3), "abc... [+3 bytes truncados]");
        // Não parte o caractere de 2 bytes ('ã' ocupa os bytes 1 e 2).
        assert_eq!(clip("aãb", 2), "a... [+3 bytes truncados]");
    }

    #[test]
    fn formata_linha() {
        assert_eq!(
            line(0, Lvl::Warn, "oi"),
            "1970-01-01T00:00:00.000Z WARN  oi\n"
        );
        assert_eq!(
            line(0, Lvl::Error, "a\r\nb"),
            "1970-01-01T00:00:00.000Z ERROR a\n    b\n"
        );
        let l = line(0, Lvl::Info, &"x".repeat(MAX_MSG + 10));
        assert!(l.contains("[+10 bytes truncados]"));
        assert!(l.len() < MAX_MSG + 100);
    }

    #[test]
    fn rotacao() {
        assert_eq!(file_name(0), "kapture.log");
        assert_eq!(file_name(2), "kapture.2.log");
        assert_eq!(
            rotation(3),
            vec![
                Step::Remove("kapture.2.log".into()),
                Step::Rename("kapture.1.log".into(), "kapture.2.log".into()),
                Step::Rename("kapture.log".into(), "kapture.1.log".into()),
            ]
        );
        assert_eq!(rotation(1), vec![Step::Remove("kapture.log".into())]);
        assert_eq!(rotation(0), rotation(1));
        assert!(!needs_rotation(0, 10_000_000, MAX_BYTES));
        assert!(!needs_rotation(100, 100, 200));
        assert!(needs_rotation(100, 101, 200));
    }

    #[test]
    fn textos_auxiliares() {
        assert_eq!(hresult(-2147024891), "0x80070005");
        assert_eq!(hresult(1), "0x00000001");
        assert_eq!(
            logs_dir(Some("C:\\Users\\a\\AppData\\Local"), "T"),
            "C:\\Users\\a\\AppData\\Local\\kapture\\logs"
        );
        assert_eq!(logs_dir(None, "C:\\Temp\\"), "C:\\Temp\\kapture\\logs");
        assert_eq!(logs_dir(Some(""), "T"), "T\\kapture\\logs");
        assert_eq!(
            start_line("1.0", "10.0.1", &["a".into()]),
            "kapture 1.0 iniciado; Windows 10.0.1; args: [\"a\"]"
        );
        assert_eq!(
            panic_text("boom", Some(("a.rs", 3)), "main", "bt"),
            "PANICO na thread 'main' em a.rs:3: boom\nbt"
        );
        assert!(panic_text("x", None, "t", "").contains("local desconhecido"));
    }

    #[test]
    fn cfg_sem_id_do_microfone() {
        let mut c = RecCfg {
            mode: Mode::Area,
            target_id: Some(7),
            region: Some(Region {
                x: 1,
                y: 2,
                w: 3,
                h: 4,
            }),
            mic_id: Some("segredo".into()),
            sys: true,
            fps: 30,
            dir: None,
        };
        let s = cfg_line(&c);
        assert!(s.contains("modo=Area") && s.contains("mic=sim") && s.contains("fps=30"));
        assert!(!s.contains("segredo"));
        c.mic_id = Some(String::new());
        assert!(cfg_line(&c).contains("mic=nao"));
        c.mic_id = None;
        assert!(cfg_line(&c).contains("mic=nao"));
    }

    #[test]
    fn payload_de_panico() {
        assert_eq!(payload_str(&"a"), "a");
        assert_eq!(payload_str(&String::from("b")), "b");
        assert_eq!(payload_str(&5u8), "(sem mensagem)");
    }

    #[test]
    fn ipc_e_ui() {
        let l = FakeLog::default();
        log_ipc(&l, "start_rec", 12, &Ok::<(), String>(()));
        log_ipc(&l, "stop_rec", 3, &Err::<(), String>("falhou".into()));
        log_ipc(&l, "x", 1, &Err::<u8, &str>("e"));
        log_ipc(&l, "y", 1, &Ok::<u8, &str>(1));
        log_ui(&l, "error", "quebrou");
        log_ui(&l, "???", "ok");
        assert_eq!(
            l.all(),
            vec![
                (Lvl::Info, "ipc start_rec: ok (12 ms)".to_string()),
                (Lvl::Error, "ipc stop_rec: erro: falhou (3 ms)".to_string()),
                (Lvl::Error, "ipc x: erro: e (1 ms)".to_string()),
                (Lvl::Info, "ipc y: ok (1 ms)".to_string()),
                (Lvl::Error, "[ui] quebrou".to_string()),
                (Lvl::Info, "[ui] ok".to_string()),
            ]
        );
    }
}
