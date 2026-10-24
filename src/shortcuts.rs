/// One source for menu accelerators and the read-only shortcuts table.
pub struct Shortcut {
    pub action: &'static str,
    pub label: &'static str,
    pub accelerators: &'static [&'static str],
    pub display: &'static str,
}
pub const SHORTCUTS: &[Shortcut] = &[
    Shortcut {
        action: "show-explorer",
        label: "show-explorer",
        accelerators: &["<Primary>b"],
        display: "Ctrl+B",
    },
    Shortcut {
        action: "new",
        label: "new-file",
        accelerators: &["<Primary>n"],
        display: "Ctrl+N",
    },
    Shortcut {
        action: "open",
        label: "open-file",
        accelerators: &["<Primary>o"],
        display: "Ctrl+O",
    },
    Shortcut {
        action: "close",
        label: "close-file",
        accelerators: &["<Primary>w"],
        display: "Ctrl+W",
    },
    Shortcut {
        action: "save",
        label: "save",
        accelerators: &["<Primary>s"],
        display: "Ctrl+S",
    },
    Shortcut {
        action: "save-as",
        label: "save-as",
        accelerators: &["<Primary><Shift>s"],
        display: "Ctrl+Shift+S",
    },
    Shortcut {
        action: "undo",
        label: "undo",
        accelerators: &["<Primary>z"],
        display: "Ctrl+Z",
    },
    Shortcut {
        action: "redo",
        label: "redo",
        accelerators: &["<Primary>y", "<Primary><Shift>z"],
        display: "Ctrl+Y / Ctrl+Shift+Z",
    },
];
pub const TEXT_SHORTCUTS: &[(&str, &str)] = &[
    ("Ctrl+C", "copy"),
    ("Ctrl+V", "paste"),
    ("Ctrl+X", "cut"),
    ("Ctrl+A", "select-all"),
    ("F2", "rename-explorer"),
    ("Delete", "delete-explorer"),
];
