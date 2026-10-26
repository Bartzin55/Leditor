use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::langid;

/// Embedded Fluent catalog: no working-directory or network dependency.
pub struct I18n(FluentBundle<FluentResource>, String);
impl Default for I18n {
    fn default() -> Self {
        let key = gtk::glib::KeyFile::new();
        let _ = key.load_from_file(
            gtk::glib::user_data_dir().join("leditor/settings.ini"),
            gtk::glib::KeyFileFlags::NONE,
        );
        Self::new(
            &key.string("settings", "language")
                .unwrap_or_else(|_| "en-US".into()),
        )
    }
}
impl I18n {
    pub fn language(&self) -> &str {
        &self.1
    }
    pub fn new(language: &str) -> Self {
        let (language, catalog) = match language {
            "pt-BR" => ("pt-BR", include_str!("../locales/pt-BR.ftl")),
            "es" => ("es", include_str!("../locales/es.ftl")),
            _ => ("en-US", include_str!("../locales/en-US.ftl")),
        };
        let resource = FluentResource::try_new(catalog.to_owned()).expect("valid catalog");
        let mut bundle = FluentBundle::new(vec![language.parse().unwrap_or(langid!("en-US"))]);
        bundle.add_resource(resource).expect("unique keys");
        Self(bundle, language.to_owned())
    }
    pub fn text(&self, id: &str) -> String {
        self.format(id, None)
    }
    pub fn format(&self, id: &str, args: Option<&FluentArgs<'_>>) -> String {
        let pattern = self
            .0
            .get_message(id)
            .and_then(|m| m.value())
            .expect("known translation key");
        let mut errors = Vec::new();
        let value = self
            .0
            .format_pattern(pattern, args, &mut errors)
            .into_owned();
        debug_assert!(errors.is_empty(), "translation error: {errors:?}");
        value
    }
}
