//! Adaptador `Updater` neutro: o kapture é distribuído só como `kapture.exe`
//! solto, então não há atualização automática. A interface cai no aviso
//! "Atualização disponível" (consulta ao GitHub), e o botão Baixar abre a
//! página do release. Serve em todos os SOs.

use cap_core::domain::UpdInfo;
use cap_core::ports::Updater;
use cap_core::Res;

/// Mensagem padrão das operações que dependem de atualização automática.
const MSG: &str = "A atualização automática não está disponível: baixe o novo kapture.exe.";

/// Atualizador que nunca atualiza sozinho.
pub struct NoUpdater;

impl Updater for NoUpdater {
    fn supported(&self) -> bool {
        false
    }

    fn check(&self) -> Res<Option<UpdInfo>> {
        Err(MSG.into())
    }

    fn download(&self, _progress: &dyn Fn(u8)) -> Res<()> {
        Err(MSG.into())
    }

    fn apply_and_restart(&self) -> Res<()> {
        Err(MSG.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nada é suportado e toda operação explica o motivo.
    #[test]
    fn never_updates() {
        let u = NoUpdater;
        assert!(!u.supported());
        assert!(u.check().is_err());
        assert!(u.download(&|_| {}).is_err());
        assert!(u.apply_and_restart().is_err());
    }
}
