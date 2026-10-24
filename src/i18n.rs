use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::langid;

/// Embedded Fluent catalog: no working-directory or network dependency.
pub struct I18n(FluentBundle<FluentResource>);
impl Default for I18n {
    fn default() -> Self {
        let resource = FluentResource::try_new(include_str!("../locales/en-US.ftl").to_owned())
            .expect("valid English catalog");
        let mut bundle = FluentBundle::new(vec![langid!("en-US")]);
        bundle.add_resource(resource).expect("unique message IDs");
        Self(bundle)
    }
}
impl I18n {
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
