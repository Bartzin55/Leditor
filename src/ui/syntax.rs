use super::*;
impl Editor {
    pub(super) fn detect_syntax(&self, tab: &Tab) {
        if !tab.is_text() {
            return;
        }
        let document = tab.document.borrow();
        let sample = document.text.as_bytes();
        let (content_type, _) = gio::content_type_guess(
            document.path.as_deref(),
            Some(&sample[..sample.len().min(4096)]),
        );
        let mime = gio::content_type_get_mime_type(&content_type);
        let language = sourceview5::LanguageManager::default()
            .guess_language(document.path.as_deref(), mime.as_deref());
        tab.source_buffer().set_language(language.as_ref());
        tab.syntax_detected.set(true);
        self.apply_syntax(tab);
    }
    pub(super) fn choose_syntax(&self, id: &str) {
        if let Some(tab) = self.current().filter(|t| t.is_text()) {
            let language = sourceview5::LanguageManager::default().language(id);
            if id != "plain" && language.is_none() {
                return;
            }
            tab.source_buffer().set_language(language.as_ref());
            tab.syntax_detected.set(true);
            self.apply_syntax(&tab);
            self.refresh();
        }
    }
    pub(super) fn apply_syntax(&self, tab: &Tab) {
        let buffer = tab.source_buffer();
        buffer.set_highlight_syntax(self.syntax_enabled.get() && tab.is_text());
        buffer.set_highlight_matching_brackets(false);
        let theme = self.theme.borrow();
        let light =
            theme.as_str() == "light" || (theme.as_str() == "system" && !self.system_dark.get());
        let manager = sourceview5::StyleSchemeManager::default();
        let scheme = manager
            .scheme(if light { "Adwaita" } else { "Adwaita-dark" })
            .or_else(|| manager.scheme(if light { "classic" } else { "oblivion" }));
        buffer.set_style_scheme(scheme.as_ref());
    }
    pub(super) fn sync_syntax_action(&self) {
        let id = self
            .current()
            .filter(|t| t.is_text())
            .and_then(|t| t.source_buffer().language())
            .map(|l| l.id().to_string())
            .unwrap_or_else(|| "plain".into());
        self.actions["syntax-language"].set_state(&id.to_variant());
    }
}
