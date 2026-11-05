use super::*;
impl Editor {
    pub(super) fn sort_selection(&self, lines: bool) {
        if self.busy.get() {
            return;
        }
        let Some(tab) = self
            .current()
            .filter(|t| t.is_text() && t.view.is_editable())
        else {
            return;
        };
        let b = tab.source_buffer();
        let Some((mut start, mut end)) = b.selection_bounds() else {
            return;
        };
        if lines {
            start.set_line_offset(0);
            // Column zero of the next line is outside the selection.
            if !end.starts_line() && !end.ends_line() {
                end.forward_to_line_end();
            }
        }
        let first = b.create_mark(None, &start, true);
        let last = b.create_mark(None, &end, false);
        let original_length = b.char_count();
        let preserve_eof = lines && end.is_end() && !end.starts_line();
        b.begin_user_action();
        if lines {
            b.sort_lines(&mut start, &mut end, sourceview5::SortFlags::empty(), 0);
            // GtkSourceView 5.12 adds a final terminator. Preserve the original EOF.
            if preserve_eof && b.char_count() == original_length + 1 {
                let mut end = b.end_iter();
                let mut start = end;
                if start.backward_char() && start.char() == '\n' {
                    b.delete(&mut start, &mut end);
                }
            }
        } else {
            let text = b.text(&start, &end, true);
            let mut chars: Vec<String> = text.chars().map(|c| c.to_string()).collect();
            chars.sort_by_cached_key(|c| glib::CollationKey::from(glib::casefold(c)));
            let sorted = chars.concat();
            if text.as_str() != sorted {
                b.delete(&mut start, &mut end);
                b.insert(&mut start, &sorted);
            }
        }
        b.select_range(&b.iter_at_mark(&last), &b.iter_at_mark(&first));
        b.delete_mark(&first);
        b.delete_mark(&last);
        b.end_user_action();
        tab.view.scroll_mark_onscreen(&b.get_insert());
    }
    pub(super) fn refresh_sort_actions(&self) {
        let enabled = !self.busy.get()
            && self
                .current()
                .is_some_and(|t| t.is_text() && t.view.buffer().has_selection());
        for name in ["sort-lines", "sort-characters"] {
            self.actions[name].set_enabled(enabled);
        }
    }
    /// Trim the live buffer, keeping disk, recovery and the native undo stack consistent.
    pub(super) fn trim_for_save(&self, tab: &Tab, autosave: bool) {
        if !self.trim_trailing.get() || !tab.is_text() {
            return;
        }
        let b = tab.source_buffer();
        // Pausing between words with autosave active must not erase the current space.
        let skip = (autosave && self.active.get() == Some(tab.id) && tab.view.has_focus())
            .then(|| b.iter_at_offset(b.cursor_position()).line());
        let text = b.text(&b.start_iter(), &b.end_iter(), true);
        let ranges = crate::text_ops::trailing_ranges(&text, skip);
        if ranges.is_empty() {
            return;
        }
        b.begin_user_action();
        for (start, end) in ranges.into_iter().rev() {
            b.delete(&mut b.iter_at_offset(start), &mut b.iter_at_offset(end));
        }
        b.end_user_action();
    }
}
