//! Montagem da URL da janela da webcam (`webcam.html?dev=<id>`).

/// Codifica um valor para uso em query string (percent-encoding RFC 3986:
/// só letras, dígitos e `-._~` passam crus).
pub fn enc_query(v: &str) -> String {
    let mut out = String::with_capacity(v.len());
    for b in v.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Caminho relativo da página da webcam, com `?dev=` se houver câmera escolhida.
pub fn cam_page(dev_id: Option<&str>) -> String {
    match dev_id {
        Some(d) if !d.is_empty() => format!("webcam.html?dev={}", enc_query(d)),
        _ => "webcam.html".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codifica_caracteres_especiais() {
        assert_eq!(enc_query("abc-1._~"), "abc-1._~");
        assert_eq!(enc_query("a b&c"), "a%20b%26c");
        assert_eq!(
            enc_query("\\\\?\\usb#vid{1}"),
            "%5C%5C%3F%5Cusb%23vid%7B1%7D"
        );
        assert_eq!(enc_query("ç"), "%C3%A7");
    }

    #[test]
    fn pagina_da_webcam() {
        assert_eq!(cam_page(None), "webcam.html");
        assert_eq!(cam_page(Some("")), "webcam.html");
        assert_eq!(cam_page(Some("x y")), "webcam.html?dev=x%20y");
    }
}
