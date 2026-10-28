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

/// Preserve literal tabs/spaces, stopping at the first non-indentation character.
pub fn leading_indent(text: &str) -> &str {
    &text[..text
        .bytes()
        .take_while(|b| *b == b' ' || *b == b'\t')
        .count()]
}
pub fn indent_columns(text: &str, width: usize) -> usize {
    let width = width.max(1);
    leading_indent(text).bytes().fold(0, |col, byte| {
        if byte == b'\t' {
            col + width - col % width
        } else {
            col + 1
        }
    })
}
#[cfg(test)]
mod indentation_tests {
    use super::*;
    #[test]
    fn preserves_mixed_indent_and_unicode() {
        assert_eq!(leading_indent(" \t  ação"), " \t  ");
        assert_eq!(leading_indent("rust"), "");
        assert_eq!(leading_indent("    "), "    ");
        assert_eq!(leading_indent("\n"), "");
    }
    #[test]
    fn tab_stops_respect_width() {
        assert_eq!(indent_columns(" \t x", 4), 5);
        assert_eq!(indent_columns("\t\t", 2), 4);
        assert_eq!(indent_columns("   x", 8), 3);
    }
}

/// Blank lines continue the deepest adjacent indentation without changing text.
pub fn guide_columns(text: &str, width: usize) -> Vec<usize> {
    let lines: Vec<_> = text.split('\n').collect();
    let mut depths: Vec<_> = lines
        .iter()
        .map(|line| indent_columns(leading_indent(line), width.max(1)))
        .collect();
    let mut previous = 0;
    for (line, depth) in lines.iter().zip(&mut depths) {
        if line.trim().is_empty() {
            *depth = (*depth).max(previous);
        } else {
            previous = *depth;
        }
    }
    let mut next = 0;
    for (line, depth) in lines.iter().zip(&mut depths).rev() {
        if line.trim().is_empty() {
            *depth = (*depth).max(next);
        } else {
            next = *depth;
        }
    }
    depths
}
#[cfg(test)]
mod guide_tests {
    #[test]
    fn blank_lines_keep_context_and_tabs() {
        assert_eq!(
            super::guide_columns("root\n    one\n\n\t\ttwo\n\n    three\n\nroot", 4),
            vec![0, 4, 8, 8, 8, 4, 4, 0]
        );
        assert_eq!(super::guide_columns("\n  text\n", 2), vec![2, 2, 2]);
    }
}
