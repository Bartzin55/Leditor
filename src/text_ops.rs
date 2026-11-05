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

/// GTK character offsets of ASCII spaces/tabs at line ends. Preserve newlines.
pub fn trailing_ranges(text: &str, skip_line: Option<i32>) -> Vec<(i32, i32)> {
    let mut ranges = Vec::new();
    let mut offset = 0;
    for (line, value) in text.split('\n').enumerate() {
        let length = value.chars().count() as i32;
        // Mixed LF/CRLF documents are intentionally not normalized by Document.
        let content = value.strip_suffix('\r').unwrap_or(value);
        let content_length = content.chars().count() as i32;
        let count = content
            .chars()
            .rev()
            .take_while(|c| matches!(c, ' ' | '\t'))
            .count() as i32;
        if count > 0 && skip_line != Some(line as i32) {
            ranges.push((offset + content_length - count, offset + content_length));
        }
        offset += length + 1;
    }
    ranges
}
#[cfg(test)]
mod trailing_tests {
    #[test]
    fn unicode_offsets_and_preserved_line_endings() {
        assert_eq!(
            super::trailing_ranges("ação \t\n \t\nx\u{a0}\nlast  ", None),
            vec![(4, 6), (7, 9), (17, 19)]
        );
        assert_eq!(super::trailing_ranges("a \nb\t\n", Some(0)), vec![(4, 5)]);
        assert_eq!(
            super::trailing_ranges("a \r\nb\t\n", None),
            vec![(1, 2), (5, 6)]
        );
        assert!(super::trailing_ranges("", None).is_empty());
        assert!(super::trailing_ranges("a\nb\n", None).is_empty());
    }
}
