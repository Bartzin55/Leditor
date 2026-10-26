pub fn convert(text: &str, mode: &str) -> String {
    match mode {
        "case-upper" => text.to_uppercase(),
        "case-lower" => text.to_lowercase(),
        _ => {
            let mut first = true;
            text.chars()
                .map(|c| {
                    if c.is_alphabetic() && first {
                        first = false;
                        c.to_uppercase().collect::<String>()
                    } else {
                        if mode == "case-words" && !c.is_alphanumeric() && c != '_' {
                            first = true;
                        }
                        c.to_string()
                    }
                })
                .collect()
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_cases() {
        assert_eq!(convert("ação española", "case-upper"), "AÇÃO ESPAÑOLA");
        assert_eq!(convert("éCOLE", "case-first"), "ÉCOLE");
        assert_eq!(
            convert(" olá-mundo mañana", "case-words"),
            " Olá-Mundo Mañana"
        );
        assert_eq!(convert("Straße", "case-upper"), "STRASSE");
    }
}
