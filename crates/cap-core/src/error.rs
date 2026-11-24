//! Erro único do núcleo: uma mensagem em português para o usuário.

use std::fmt;

/// Erro com mensagem já pronta para exibição.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapErr(pub String);

/// Resultado padrão do núcleo.
pub type Res<T> = Result<T, CapErr>;

impl fmt::Display for CapErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CapErr {}

impl From<&str> for CapErr {
    fn from(s: &str) -> Self {
        CapErr(s.to_string())
    }
}

impl From<String> for CapErr {
    fn from(s: String) -> Self {
        CapErr(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversoes_e_display() {
        let a: CapErr = "falhou".into();
        let b: CapErr = String::from("falhou").into();
        assert_eq!(a, b);
        assert_eq!(a.to_string(), "falhou");
        let dynamic: &dyn std::error::Error = &a;
        assert_eq!(dynamic.to_string(), "falhou");
    }
}
