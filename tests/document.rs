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
