use leditor::{document::Document, file_ops::*};
use std::fs;
#[test]
fn create_and_conflicts_never_replace() {
    let root = tempfile::tempdir().unwrap();
    let file = create(root.path(), "hello.txt", false).unwrap();
    fs::write(&file, "keep").unwrap();
    assert!(create(root.path(), "hello.txt", false).is_err());
    assert!(create(root.path(), "hello.txt", true).is_err());
    assert_eq!(fs::read_to_string(&file).unwrap(), "keep");
}
#[test]
fn invalid_names_cannot_escape_parent() {
    let root = tempfile::tempdir().unwrap();
    for name in [
        "",
        ".",
        "..",
        "../escape",
        "a/b",
        "a\\b",
        "/absolute",
        "C:drive",
        "CON",
        "NUL.txt",
        "COM1",
        "LPT9.txt",
        "trailing.",
        "trailing ",
        "line\nfeed",
    ] {
        assert!(child(root.path(), name).is_err(), "{name:?}");
    }
    assert!(child(root.path(), "Olá 世界.txt").is_ok());
}
#[test]
fn recursive_copy_is_independent_and_preserves_source() {
    let root = tempfile::tempdir().unwrap();
    let source = create(root.path(), "source", true).unwrap();
    let nested = create(&source, "nested", true).unwrap();
    fs::write(nested.join("file"), b"abc\0\xff").unwrap();
    let target = root.path().join("copy");
    copy_entry(&source, &target).unwrap();
    assert_eq!(fs::read(target.join("nested/file")).unwrap(), b"abc\0\xff");
    fs::write(target.join("nested/file"), "new").unwrap();
    assert_eq!(fs::read(nested.join("file")).unwrap(), b"abc\0\xff");
    assert!(copy_entry(&source, &target).is_err());
    assert_eq!(
        fs::read_to_string(target.join("nested/file")).unwrap(),
        "new"
    );
}
#[test]
fn move_and_rename_preserve_bytes_and_refuse_conflicts() {
    let root = tempfile::tempdir().unwrap();
    let a = root.path().join("a");
    let b = root.path().join("b");
    fs::write(&a, "source").unwrap();
    fs::write(&b, "destination").unwrap();
    assert!(move_entry(&a, &b).is_err());
    assert_eq!(fs::read_to_string(&a).unwrap(), "source");
    assert_eq!(fs::read_to_string(&b).unwrap(), "destination");
    let c = root.path().join("c");
    move_entry(&a, &c).unwrap();
    assert!(!a.exists());
    assert_eq!(fs::read_to_string(c).unwrap(), "source");
}
#[test]
fn folder_cannot_be_placed_in_itself_or_descendants() {
    let root = tempfile::tempdir().unwrap();
    let folder = create(root.path(), "folder", true).unwrap();
    let nested = create(&folder, "nested", true).unwrap();
    for target in [folder.join("copy"), nested.join("copy")] {
        assert!(copy_entry(&folder, &target).is_err());
        assert!(move_entry(&folder, &target).is_err());
        assert!(!target.exists());
        assert!(nested.exists());
    }
}
#[test]
fn deleting_detached_document_keeps_recoverable_text() {
    let root = tempfile::tempdir().unwrap();
    let path = create(root.path(), "file", false).unwrap();
    let mut doc = Document::open(&path).unwrap();
    doc.text = "unsaved work".into();
    doc.detach();
    delete(&path).unwrap();
    assert!(doc.path.is_none());
    assert!(doc.is_dirty());
    let saved = root.path().join("recovered");
    doc.save_to(&saved).unwrap();
    assert_eq!(fs::read_to_string(saved).unwrap(), "unsaved work");
    let mut empty = Document::default();
    empty.detach();
    assert!(empty.is_dirty());
}
#[test]
fn path_mapping_obeys_component_boundaries() {
    let root = tempfile::tempdir().unwrap();
    let a = create(root.path(), "a", true).unwrap();
    let ab = create(root.path(), "ab", true).unwrap();
    assert_eq!(
        relative(&a.join("nested/file"), &a).unwrap(),
        std::path::Path::new("nested/file")
    );
    assert!(relative(&ab, &a).is_none());
}
#[cfg(unix)]
#[test]
fn symlink_copy_failure_leaves_no_partial_destination() {
    let root = tempfile::tempdir().unwrap();
    let source = create(root.path(), "source", true).unwrap();
    fs::write(source.join("first"), "keep").unwrap();
    std::os::unix::fs::symlink(&source, source.join("cycle")).unwrap();
    let dest = root.path().join("copy");
    assert!(copy_entry(&source, &dest).is_err());
    assert!(!dest.exists());
    assert!(source.join("first").exists());
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
}
#[cfg(unix)]
#[test]
fn delete_symlink_does_not_delete_target() {
    let root = tempfile::tempdir().unwrap();
    let folder = create(root.path(), "folder", true).unwrap();
    fs::write(folder.join("file"), "keep").unwrap();
    let link = root.path().join("link");
    std::os::unix::fs::symlink(&folder, &link).unwrap();
    delete(&link).unwrap();
    assert!(folder.join("file").exists());
}
