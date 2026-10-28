use super::*;

pub(super) fn build(
    window: &gtk::ApplicationWindow,
    strings: &I18n,
) -> (gio::Menu, BTreeMap<&'static str, gio::SimpleAction>) {
    let model = gio::Menu::new();
    let mut actions = BTreeMap::new();
    for (menu_name, entries) in [
        (
            "file",
            vec![
                "new",
                "open",
                "close",
                "new-folder",
                "open-folder",
                "close-folder",
                "save",
                "save-as",
                "save-all",
            ],
        ),
        (
            "edit",
            vec![
                "copy",
                "cut",
                "paste",
                "undo",
                "redo",
                "find-replace",
                "auto-indent",
                "convert-tabs",
                "number-spaces",
                "insert-above",
                "insert-below",
                "case-first",
                "case-words",
                "case-upper",
                "case-lower",
            ],
        ),
        (
            "view",
            vec![
                "word-wrap",
                "indent-guides",
                "zoom",
                "zoom-in",
                "zoom-out",
                "zoom-reset",
                "syntax-enable",
                "syntax-detect",
                "syntax-language",
            ],
        ),
        (
            "window",
            vec!["minimap", "focus", "line-numbers", "show-explorer", "theme"],
        ),
        (
            "other",
            vec![
                "enable-autosave",
                "restore-session",
                "keyboard",
                "language",
                "about",
            ],
        ),
    ] {
        let submenu = gio::Menu::new();
        let cases = gio::Menu::new();
        let indentation = gio::Menu::new();
        let syntax = gio::Menu::new();
        for name in entries {
            let action = match name {
                "theme" | "syntax-language" => gio::SimpleAction::new_stateful(
                    name,
                    Some(glib::VariantTy::STRING),
                    &if name == "theme" { "system" } else { "plain" }.to_variant(),
                ),
                "minimap" | "focus" | "indent-guides" | "word-wrap" | "convert-tabs"
                | "enable-autosave" | "restore-session" => {
                    gio::SimpleAction::new_stateful(name, None, &false.to_variant())
                }
                "auto-indent" | "line-numbers" | "show-explorer" | "syntax-enable" => {
                    gio::SimpleAction::new_stateful(name, None, &true.to_variant())
                }
                _ => gio::SimpleAction::new(name, None),
            };
            window.add_action(&action);
            actions.insert(name, action);
            let label = match name {
                "new" => "new-file",
                "open" => "open-file",
                "close" => "close-file",
                "line-numbers" => "line-counter",
                "keyboard" => "keyboard-shortcuts",
                _ => name,
            };
            let target = match name {
                "auto-indent" | "convert-tabs" | "number-spaces" => &indentation,
                "case-first" | "case-words" | "case-upper" | "case-lower" => &cases,
                "syntax-enable" | "syntax-detect" | "syntax-language" => &syntax,
                _ => &submenu,
            };
            if name == "theme" {
                let themes = gio::Menu::new();
                for (value, label) in [
                    ("system", "system-theme"),
                    ("light", "light-theme"),
                    ("dark", "dark-theme"),
                ] {
                    themes.append(
                        Some(&strings.text(label)),
                        Some(&format!("win.theme::{value}")),
                    );
                }
                submenu.append_submenu(Some(&strings.text("theme")), &themes);
            } else if name == "syntax-language" {
                let languages = gio::Menu::new();
                languages.append(
                    Some(&strings.text("plain-text")),
                    Some("win.syntax-language::plain"),
                );
                let manager = sourceview5::LanguageManager::default();
                let mut items: Vec<_> = manager
                    .language_ids()
                    .iter()
                    .filter_map(|id| manager.language(id))
                    .collect();
                items.sort_by_key(|l| l.name().to_lowercase());
                for language in items {
                    let item = gio::MenuItem::new(Some(&language.name()), None);
                    item.set_action_and_target_value(
                        Some("win.syntax-language"),
                        Some(&language.id().to_variant()),
                    );
                    languages.append_item(&item);
                }
                syntax.append_section(None, &languages);
            } else if name == "zoom" || name == "number-spaces" {
                let item = gio::MenuItem::new(None, None);
                item.set_attribute_value("custom", Some(&name.to_variant()));
                target.append_item(&item);
            } else {
                target.append(Some(&strings.text(label)), Some(&format!("win.{name}")));
            }
            if name == "number-spaces" {
                submenu.append_submenu(Some(&strings.text("indentation-options")), &indentation);
            }
        }
        if menu_name == "edit" {
            submenu.append_submenu(Some(&strings.text("convert-case")), &cases);
        }
        if menu_name == "view" {
            submenu.append_submenu(Some(&strings.text("syntax-highlighting")), &syntax);
        }
        model.append_submenu(Some(&strings.text(menu_name)), &submenu);
    }
    (model, actions)
}
