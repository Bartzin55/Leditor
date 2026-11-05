use super::*;

// GtkSourceView 5.12 discovers resource URIs but its style parser requires real
// files. Keep embedded overrides in a private temporary directory for the GUI
// thread's lifetime; no installation paths or extra runtime assets are needed.
thread_local! {
    static STYLE_DIRECTORY: Option<tempfile::TempDir> = {
        let create = || -> std::io::Result<tempfile::TempDir> {
            let dir = tempfile::Builder::new().prefix("leditor-styles-").tempdir()?;
            for (name, data) in [
                ("leditor-light.xml", include_str!("../../assets/leditor-light.xml")),
                ("leditor-dark.xml", include_str!("../../assets/leditor-dark.xml")),
            ] {
                std::fs::write(dir.path().join(name), data)?;
            }
            sourceview5::StyleSchemeManager::default()
                .append_search_path(&dir.path().to_string_lossy());
            Ok(dir)
        };
        match create() {
            Ok(dir) => Some(dir),
            Err(error) => {
                eprintln!("Leditor: could not prepare editor styles: {error}");
                None
            }
        }
    };
}

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
        STYLE_DIRECTORY.with(|_| {});
        let manager = sourceview5::StyleSchemeManager::default();
        let scheme = manager
            .scheme(if light {
                "leditor-light"
            } else {
                "leditor-dark"
            })
            .or_else(|| manager.scheme(if light { "Adwaita" } else { "Adwaita-dark" }))
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
