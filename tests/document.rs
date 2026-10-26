use leditor::{document::Document, i18n::I18n};
use std::fs;

#[test]
fn new_save_edit_save_as_and_revert() {
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("first.txt");
    let second = dir.path().join("second.txt");
    let mut doc = Document::default();
    assert!(!doc.is_dirty());
    assert!(doc.path.is_none());
    doc.text = "Olá, 世界!\n".into();
    assert!(doc.is_dirty());
    doc.save_to(&first).unwrap();
    assert!(!doc.is_dirty());
    doc.text.push('X');
    assert!(doc.is_dirty());
    doc.text.pop();
    assert!(!doc.is_dirty());
    doc.text.push('Y');
    doc.save_to(&second).unwrap();
    assert_eq!(doc.path.as_deref(), Some(second.as_path()));
    assert_eq!(fs::read_to_string(first).unwrap(), "Olá, 世界!\n");
    assert_eq!(fs::read_to_string(second).unwrap(), "Olá, 世界!\nY");
}

#[test]
fn failed_save_preserves_path_text_and_dirty_state() {
    let dir = tempfile::tempdir().unwrap();
    let original = dir.path().join("original.txt");
    fs::write(&original, "original").unwrap();
    let mut doc = Document::open(&original).unwrap();
    doc.text = "edited".into();
    assert!(doc.save_to(&dir.path().join("missing/file.txt")).is_err());
    assert_eq!(doc.path.as_deref(), Some(original.as_path()));
    assert_eq!(doc.text, "edited");
    assert!(doc.is_dirty());
    assert_eq!(fs::read_to_string(original).unwrap(), "original");
}

#[test]
fn preserves_bom_windows_newlines_and_empty_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("windows.txt");
    fs::write(&path, b"\xef\xbb\xbfhello\r\nworld\r\n").unwrap();
    let mut doc = Document::open(&path).unwrap();
    assert_eq!(doc.text, "hello\nworld\n");
    doc.text.push_str("third\n");
    doc.save_to(&path).unwrap();
    assert_eq!(
        fs::read(&path).unwrap(),
        b"\xef\xbb\xbfhello\r\nworld\r\nthird\r\n"
    );
    fs::write(&path, []).unwrap();
    let mut doc = Document::open(&path).unwrap();
    doc.save_to(&path).unwrap();
    assert!(fs::read(path).unwrap().is_empty());
}

#[test]
fn rejects_invalid_encoding_binary_and_missing_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("invalid.txt");
    assert!(Document::open(&path).is_err());
    for bytes in [&[0xff, 0xfe][..], &b"a\0b"[..]] {
        fs::write(&path, bytes).unwrap();
        assert!(Document::open(&path).is_err());
    }
}

#[test]
fn preserves_mixed_line_endings_on_unchanged_save() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mixed.txt");
    let bytes = b"one\r\ntwo\nthree\rfour";
    fs::write(&path, bytes).unwrap();
    Document::open(&path).unwrap().save_to(&path).unwrap();
    assert_eq!(fs::read(path).unwrap(), bytes);
}

#[cfg(unix)]
#[test]
fn retains_symlink_and_unix_permissions() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.txt");
    let link = dir.path().join("link.txt");
    fs::write(&target, "before").unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).unwrap();
    symlink(&target, &link).unwrap();
    let mut doc = Document::open(&link).unwrap();
    doc.text = "after".into();
    doc.save_to(&link).unwrap();
    assert!(link.is_symlink());
    assert_eq!(fs::read_to_string(target).unwrap(), "after");
    assert_eq!(
        fs::metadata(link).unwrap().permissions().mode() & 0o777,
        0o640
    );
}

#[test]
fn all_interface_messages_resolve() {
    let strings = I18n::default();
    for key in [
        "file",
        "new-file",
        "open-file",
        "close-file",
        "save",
        "save-as",
        "cancel",
        "dont-save",
        "untitled",
        "unsaved",
        "error-title",
        "close",
    ] {
        assert!(!strings.text(key).is_empty());
    }
}

#[test]
fn failed_atomic_replacement_does_not_commit_document() {
    let directory = tempfile::tempdir().unwrap();
    let mut document = Document::default();
    document.text = "Do not lose this text".into();
    // The temporary write succeeds, but a directory cannot be replaced by a file.
    assert!(document.save_to(directory.path()).is_err());
    assert!(directory.path().is_dir());
    assert!(document.path.is_none());
    assert!(document.is_dirty());
    assert_eq!(document.text, "Do not lose this text");
}

#[test]
fn file_identity_resolves_relative_segments_and_symlinks() {
    use leditor::document::same_path;
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first.txt");
    let second = directory.path().join("second.txt");
    fs::write(&first, "a").unwrap();
    fs::write(&second, "b").unwrap();
    assert!(same_path(&first, &directory.path().join("./first.txt")));
    assert!(!same_path(&first, &second));
    #[cfg(unix)]
    {
        let alias = directory.path().join("alias.txt");
        std::os::unix::fs::symlink(&first, &alias).unwrap();
        assert!(same_path(&first, &alias));
    }
}

#[test]
fn autosave_detects_conflicts_and_keeps_dirty_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("autosave.txt");
    std::fs::write(&path, "original").unwrap();
    let mut doc = leditor::document::Document::open(&path).unwrap();
    doc.text = "edit".into();
    doc.autosave().unwrap();
    assert!(!doc.is_dirty());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "edit");
    std::fs::write(&path, "external").unwrap();
    doc.text = "my text".into();
    assert!(doc.autosave().is_err());
    assert!(doc.is_dirty());
    assert_eq!(doc.text, "my text");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "external");
    std::fs::remove_file(&path).unwrap();
    assert!(doc.autosave().is_err());
    assert!(!path.exists());
}

#[test]
fn in_flight_save_keeps_new_edits_dirty_and_recovery_preserves_format() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bom-crlf.txt");
    std::fs::write(&path, b"\xef\xbb\xbffirst\r\n").unwrap();
    let mut doc = leditor::document::Document::open(&path).unwrap();
    doc.text.push_str("second\n");
    let key = gtk::glib::KeyFile::new();
    doc.write_recovery(&key, "doc");
    let mut restored = leditor::document::Document::from_recovery(&key, "doc").unwrap();
    restored.autosave().unwrap();
    doc.text.push_str("third\n");
    doc.accept_saved_snapshot(&restored);
    assert!(doc.is_dirty());
    assert!(doc.text.ends_with("third\n"));
    assert_eq!(
        std::fs::read(path).unwrap(),
        b"\xef\xbb\xbffirst\r\nsecond\r\n"
    );
}

#[test]
fn all_languages_have_matching_messages_and_variables() {
    use fluent_bundle::FluentArgs;
    use leditor::i18n::I18n;
    let en = include_str!("../locales/en-US.ftl");
    for (id, catalog) in [
        ("pt-BR", include_str!("../locales/pt-BR.ftl")),
        ("es", include_str!("../locales/es.ftl")),
        ("en-US", en),
    ] {
        let keys = |s: &str| {
            s.lines()
                .filter_map(|l| l.split_once(" = ").map(|(k, _)| k.to_owned()))
                .collect::<std::collections::BTreeSet<_>>()
        };
        assert_eq!(keys(en), keys(catalog));
        let strings = I18n::new(id);
        let mut args = FluentArgs::new();
        for name in ["name", "reason", "encoding", "format", "zoom", "language"] {
            args.set(name, "test");
        }
        for name in ["lines", "chars", "line", "column"] {
            args.set(name, 1);
        }
        for key in keys(en) {
            assert!(!strings.format(&key, Some(&args)).is_empty());
        }
    }
}
