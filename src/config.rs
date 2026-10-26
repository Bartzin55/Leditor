//! XDG preferences with read-only fallback to the pre-1.3 location.
use gtk::glib;
use std::path::PathBuf;
pub fn settings_path() -> PathBuf {
    glib::user_config_dir().join("leditor/settings.ini")
}
pub fn load() -> Result<glib::KeyFile, glib::Error> {
    load_paths(
        settings_path(),
        glib::user_data_dir().join("leditor/settings.ini"),
    )
}
fn load_paths(current: PathBuf, legacy: PathBuf) -> Result<glib::KeyFile, glib::Error> {
    let path = if current.exists() { current } else { legacy };
    let key = glib::KeyFile::new();
    key.load_from_file(path, glib::KeyFileFlags::NONE)?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_fallback_and_current_precedence() {
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("new.ini");
        let legacy = dir.path().join("old.ini");
        std::fs::write(&legacy, "[settings]\nlanguage=pt-BR\nlight=true\n").unwrap();
        assert_eq!(
            load_paths(current.clone(), legacy.clone())
                .unwrap()
                .string("settings", "language")
                .unwrap(),
            "pt-BR"
        );
        assert!(!current.exists());
        std::fs::write(&current, "[settings]\nlanguage=es\n").unwrap();
        assert_eq!(
            load_paths(current.clone(), legacy.clone())
                .unwrap()
                .string("settings", "language")
                .unwrap(),
            "es"
        );
        std::fs::write(&current, "invalid data").unwrap();
        assert!(load_paths(current, legacy.clone()).is_err());
        assert!(legacy.exists());
    }
}
