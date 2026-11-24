//! Implementação REAL de `ports::Log`: arquivo `kapture.log` com rotação.
//! Só usa a biblioteca padrão (sem `log`/`tracing`); cada linha é gravada
//! e descarregada (`flush`) na hora, para sobreviver a um aborto.

use crate::lk;
use crate::log::{
    fallback_note, file_name, line, needs_rotation, rotation, Lvl, Step, KEEP, MAX_BYTES,
};
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

/// Testa se `dir` serve: cria a pasta e abre o arquivo ativo para anexar.
fn probe(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("criar a pasta: {e}"))?;
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(file_name(0)))
        .map(|_| ())
        .map_err(|e| format!("abrir o arquivo: {e}"))
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

    /// Escolhe a primeira pasta onde o arquivo pode ser criado e aberto para anexar e devolve o
    /// log nela, mais as notas de cada pasta descartada (para a primeira linha da sessão).
    /// Se nenhuma servir, usa a última (o log fica mudo) e a nota diz isso.
    pub fn first_usable(cands: &[PathBuf]) -> (FileLog, Vec<String>) {
        let mut notes = Vec::new();
        let mut failed: Option<(&PathBuf, String)> = None;
        for d in cands {
            match probe(d) {
                Ok(()) => {
                    if let Some((t, e)) = failed {
                        notes.push(fallback_note(
                            &t.display().to_string(),
                            &e,
                            &d.display().to_string(),
                        ));
                    }
                    return (FileLog::new(d), notes);
                }
                Err(e) => {
                    if let Some((t, e0)) = failed.take() {
                        notes.push(fallback_note(
                            &t.display().to_string(),
                            &e0,
                            &d.display().to_string(),
                        ));
                    }
                    failed = Some((d, e));
                }
            }
        }
        let last = cands.last().cloned().unwrap_or_default();
        notes.push(format!(
            "nenhuma pasta de log pôde ser usada (última: '{}'); o log em disco está desligado",
            last.display()
        ));
        (FileLog::new(&last), notes)
    }

    /// Pasta onde o log grava.
    pub fn dir(&self) -> &Path {
        &self.dir
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
    fn escolhe_a_primeira_pasta_utilizavel() {
        let d = tmp("escolha");
        fs::create_dir_all(&d).unwrap();
        let ruim = d.join("arquivo");
        fs::write(&ruim, "x").unwrap();
        let boa = d.join("boa");
        // A primeira serve: sem notas.
        let (l, n) = FileLog::first_usable(&[boa.clone(), ruim.clone()]);
        assert!(n.is_empty());
        assert_eq!(l.dir(), boa.as_path());
        // A primeira falha (é um arquivo): usa a segunda e explica.
        let (l, n) = FileLog::first_usable(&[ruim.clone(), boa.clone()]);
        assert_eq!(l.dir(), boa.as_path());
        assert_eq!(n.len(), 1);
        assert!(n[0].contains("inutilizável") && n[0].contains("usando"));
        // Duas falhas e depois uma boa: uma nota por pasta descartada.
        let ruim2 = ruim.join("sub");
        let (l, n) = FileLog::first_usable(&[ruim.clone(), ruim2.clone(), boa.clone()]);
        assert_eq!(l.dir(), boa.as_path());
        assert_eq!(n.len(), 2);
        // Nenhuma serve: usa a última, muda, e diz isso.
        let (l, n) = FileLog::first_usable(&[ruim.clone(), ruim2.clone()]);
        assert_eq!(l.dir(), ruim2.as_path());
        assert!(n.last().unwrap().contains("desligado"));
        l.error("ninguém ouve");
        // Pasta existe, mas o arquivo não abre (há uma pasta com o nome do log).
        let trava = d.join("trava");
        fs::create_dir_all(trava.join("kapture.log")).unwrap();
        let (l, n) = FileLog::first_usable(&[trava.clone(), boa.clone()]);
        assert_eq!(l.dir(), boa.as_path());
        assert!(n[0].contains("abrir o arquivo"));
        // Lista vazia também não derruba.
        let (_, n) = FileLog::first_usable(&[]);
        assert_eq!(n.len(), 1);
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
