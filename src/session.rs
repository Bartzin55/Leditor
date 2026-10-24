//! Atomic, versioned recovery snapshots. GTK KeyFile avoids another dependency.
use gtk::{gio, glib, prelude::*};
use leditor::document::Document;
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

pub fn directory() -> PathBuf {
    #[cfg(test)]
    {
        std::env::temp_dir().join(format!("leditor-tests-{}", std::process::id()))
    }
    #[cfg(not(test))]
    {
        glib::user_data_dir().join("leditor")
    }
}
pub fn write(path: &Path, key: &glib::KeyFile) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("Missing parent"))?;
    std::fs::create_dir_all(parent)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(key.to_data().as_bytes())?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| e.error)?;
    Ok(())
}
pub fn read(path: &Path) -> Result<glib::KeyFile, glib::Error> {
    let key = glib::KeyFile::new();
    key.load_from_file(path, glib::KeyFileFlags::NONE)?;
    Ok(key)
}
pub fn set_path(key: &glib::KeyFile, group: &str, name: &str, path: &Path) {
    key.set_string(group, name, &gio::File::for_path(path).uri());
}
pub fn path(key: &glib::KeyFile, group: &str, name: &str) -> Option<PathBuf> {
    key.string(group, name)
        .ok()
        .and_then(|s| gio::File::for_uri(&s).path())
}
pub struct Recovered {
    pub document: Document,
    pub name: Option<String>,
    pub unsupported: bool,
    pub cursor: i32,
}
pub fn recover(key: &glib::KeyFile, group: &str) -> Result<Recovered, glib::Error> {
    let mut document = Document::from_recovery(key, group)?;
    let name = key.string(group, "name").ok().map(|s| s.to_string());
    let unsupported = key.boolean(group, "unsupported")?;
    if !unsupported {
        if let Some(path) = document.path.clone() {
            match Document::open(&path) {
                Ok(disk) if !document.is_dirty() => document = disk,
                Ok(disk) if disk.saved_content() == document.saved_content() => {}
                // Never autosave recovered edits over a different on-disk version.
                _ => document.detach(),
            }
        }
    }
    Ok(Recovered {
        document,
        name,
        unsupported,
        cursor: key.integer(group, "cursor").unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot(document: &Document) -> glib::KeyFile {
        let key = glib::KeyFile::new();
        document.write_recovery(&key, "tab");
        key.set_boolean("tab", "unsupported", false);
        key
    }
    #[test]
    fn untitled_roundtrip_and_atomic_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let mut doc = Document::default();
        doc.text = "  café\n\t[section]\\\nlast".into();
        let path = dir.path().join("session.ini");
        write(&path, &snapshot(&doc)).unwrap();
        let restored = recover(&read(&path).unwrap(), "tab").unwrap();
        assert_eq!(restored.document.text, doc.text);
        assert!(restored.document.is_dirty());
        assert!(restored.document.path.is_none());
        doc.text = "new snapshot".into();
        write(&path, &snapshot(&doc)).unwrap();
        assert_eq!(
            recover(&read(&path).unwrap(), "tab").unwrap().document.text,
            doc.text
        );
    }
    #[test]
    fn conflict_or_missing_file_recovers_without_overwriting() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "original").unwrap();
        let mut doc = Document::open(&path).unwrap();
        doc.text = "local edit".into();
        let key = snapshot(&doc);
        std::fs::write(&path, "external edit").unwrap();
        let recovered = recover(&key, "tab").unwrap();
        assert!(recovered.document.path.is_none());
        assert_eq!(recovered.document.text, "local edit");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "external edit");
        std::fs::remove_file(&path).unwrap();
        assert!(recover(&key, "tab").unwrap().document.path.is_none());
    }
    #[test]
    fn clean_files_reload_current_disk_contents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "before").unwrap();
        let key = snapshot(&Document::open(&path).unwrap());
        std::fs::write(&path, "after").unwrap();
        let doc = recover(&key, "tab").unwrap().document;
        assert_eq!(doc.text, "after");
        assert!(!doc.is_dirty());
    }
    #[test]
    fn invalid_data_is_rejected_and_failed_write_preserves_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.ini");
        std::fs::write(&path, "invalid snapshot").unwrap();
        assert!(read(&path).is_err());
        assert!(recover(&glib::KeyFile::new(), "tab").is_err());
        assert!(write(&path.join("child"), &glib::KeyFile::new()).is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), "invalid snapshot");
    }
}
