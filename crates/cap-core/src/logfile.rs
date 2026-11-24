//! Implementação REAL de `ports::Log`: arquivo `kapture.log` com rotação.
//! Só usa a biblioteca padrão (sem `log`/`tracing`); cada linha é gravada
//! e descarregada (`flush`) na hora, para sobreviver a um aborto.

use crate::lk;
use crate::log::{file_name, line, needs_rotation, rotation, Lvl, Step, KEEP, MAX_BYTES};
use crate::ports::Log;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Fonte de tempo (ms desde a época Unix), injetável nos testes.
type Now = Box<dyn Fn() -> u64 + Send + Sync>;

/// Arquivo aberto e seu tamanho atual.
struct Open {
    file: Option<File>,
    size: u64,
}

/// Log em arquivo, seguro para várias threads.
pub struct FileLog {
    dir: PathBuf,
    max: u64,
    keep: usize,
    now: Now,
    st: Mutex<Open>,
}

/// Relógio do sistema em ms desde 1970 (UTC).
fn sys_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

impl FileLog {
    /// Log em `dir` com os limites padrão (512 KB, 3 arquivos).
    pub fn new(dir: &Path) -> Self {
        Self::with(dir, MAX_BYTES, KEEP, Box::new(sys_ms))
    }

    /// Log com limites e relógio explícitos (testes).
    pub fn with(dir: &Path, max: u64, keep: usize, now: Now) -> Self {
        FileLog {
            dir: dir.to_path_buf(),
            max,
            keep,
            now,
            st: Mutex::new(Open {
                file: None,
                size: 0,
            }),
        }
    }

    /// Caminho do arquivo de índice `i` (0 = ativo).
    pub fn path(&self, i: usize) -> PathBuf {
        self.dir.join(file_name(i))
    }

    /// Abre (ou cria) o arquivo ativo em modo de anexar; falhas só deixam
    /// o log desligado até a próxima tentativa.
    fn open(&self, st: &mut Open) {
        let _ = std::fs::create_dir_all(&self.dir);
        st.size = std::fs::metadata(self.path(0)).map_or(0, |m| m.len());
        st.file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path(0))
            .ok();
    }

    /// Aplica a rotação (erros ignorados: log nunca derruba o app).
    fn rotate(&self) {
        for s in rotation(self.keep) {
            let _ = match s {
                Step::Remove(n) => std::fs::remove_file(self.dir.join(n)),
                Step::Rename(a, b) => std::fs::rename(self.dir.join(a), self.dir.join(b)),
            };
        }
    }
}

impl Log for FileLog {
    fn write(&self, lvl: Lvl, msg: &str) {
        let text = line((self.now)(), lvl, msg);
        let add = text.len() as u64;
        let mut st = lk(&self.st);
        if st.file.is_none() {
            self.open(&mut st);
        }
        if needs_rotation(st.size, add, self.max) {
            // Fecha antes de renomear (o Windows não renomeia arquivo aberto).
            st.file = None;
            self.rotate();
            self.open(&mut st);
        }
        if let Some(f) = st.file.as_mut() {
            if f.write_all(text.as_bytes()).and_then(|_| f.flush()).is_ok() {
                st.size += add;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Pasta temporária exclusiva do teste.
    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("cap-log-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    fn at_zero() -> Now {
        Box::new(|| 0)
    }

    #[test]
    fn cria_pasta_e_grava_linhas() {
        let d = tmp("grava");
        let l = FileLog::with(&d, MAX_BYTES, KEEP, at_zero());
        l.info("um");
        l.error("dois");
        let s = fs::read_to_string(l.path(0)).unwrap();
        assert_eq!(
            s,
            "1970-01-01T00:00:00.000Z INFO  um\n1970-01-01T00:00:00.000Z ERROR dois\n"
        );
        // Nova instância continua anexando e conhece o tamanho existente.
        let l2 = FileLog::new(&d);
        l2.warn("tres");
        let s = fs::read_to_string(l2.path(0)).unwrap();
        assert_eq!(s.lines().count(), 3);
        assert!(s.lines().last().unwrap().contains("WARN  tres"));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn rotaciona_e_mantem_no_maximo_tres_arquivos() {
        let d = tmp("rota");
        // Cada linha tem ~40 bytes; limite de 100 força rotações frequentes.
        let l = FileLog::with(&d, 100, 3, at_zero());
        for i in 0..30 {
            l.info(&format!("linha {i:02}"));
        }
        let mut nomes: Vec<String> = fs::read_dir(&d)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        nomes.sort();
        assert_eq!(nomes, vec!["kapture.1.log", "kapture.2.log", "kapture.log"]);
        // A linha mais recente está no arquivo ativo e nenhum passa do limite.
        assert!(fs::read_to_string(l.path(0)).unwrap().contains("linha 29"));
        for i in 0..3 {
            assert!(fs::metadata(l.path(i)).unwrap().len() <= 100);
        }
        // O mais antigo foi descartado.
        let tudo: String = (0..3)
            .map(|i| fs::read_to_string(l.path(i)).unwrap())
            .collect();
        assert!(!tudo.contains("linha 00"));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn falha_ao_abrir_nao_derruba() {
        let d = tmp("falha");
        fs::create_dir_all(&d).unwrap();
        // O "diretório" de logs é um arquivo comum: nada pode ser aberto.
        let f = d.join("arquivo");
        fs::write(&f, "x").unwrap();
        let l = FileLog::new(&f);
        l.error("ninguém ouve");
        l.error("de novo");
        assert_eq!(fs::read_to_string(&f).unwrap(), "x");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn falha_ao_gravar_nao_derruba_nem_conta_bytes() {
        let d = tmp("gravar");
        let l = FileLog::with(&d, MAX_BYTES, KEEP, at_zero());
        l.info("ok");
        let antes = fs::read_to_string(l.path(0)).unwrap();
        // Troca o arquivo aberto por um somente leitura: gravar falha, em
        // qualquer sistema e mesmo como administrador.
        let size = {
            let mut st = lk(&l.st);
            st.file = Some(File::open(l.path(0)).unwrap());
            st.size
        };
        l.error("perdida");
        assert_eq!(fs::read_to_string(l.path(0)).unwrap(), antes);
        assert_eq!(lk(&l.st).size, size);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn relogio_do_sistema_e_usado_por_padrao() {
        assert!(sys_ms() > 1_700_000_000_000);
        let d = tmp("relogio");
        FileLog::new(&d).info("agora");
        let s = fs::read_to_string(d.join("kapture.log")).unwrap();
        assert!(s.starts_with("20"));
        let _ = fs::remove_dir_all(&d);
    }
}
