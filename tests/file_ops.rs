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

#[test]
fn duplicate_uses_available_number_and_preserves_extension_and_bytes() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("ação.md");
    fs::write(&source, b"\xef\xbb\xbftext\r\n").unwrap();
    fs::write(root.path().join("ação (Copy 1).md"), "keep").unwrap();
    let copy = duplicate(&source, "Copy").unwrap();
    assert_eq!(copy, root.path().join("ação (Copy 2).md"));
    assert_eq!(fs::read(&copy).unwrap(), fs::read(&source).unwrap());
    assert_eq!(
        fs::read_to_string(root.path().join("ação (Copy 1).md")).unwrap(),
        "keep"
    );
    fs::write(&copy, "independent").unwrap();
    assert_ne!(fs::read(&copy).unwrap(), fs::read(&source).unwrap());
    let dotfile = root.path().join(".env");
    fs::write(&dotfile, "keep").unwrap();
    assert_eq!(
        duplicate(&dotfile, "Cópia").unwrap(),
        root.path().join(".env (Cópia 1)")
    );
}
#[test]
fn duplicate_folder_keeps_dotted_name_and_copies_nested_files() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("folder.v1");
    fs::create_dir_all(source.join("nested")).unwrap();
    fs::write(source.join("nested/a"), b"\0\xff\n").unwrap();
    let copy = duplicate(&source, "Copy").unwrap();
    assert_eq!(copy, root.path().join("folder.v1 (Copy 1)"));
    assert_eq!(fs::read(copy.join("nested/a")).unwrap(), b"\0\xff\n");
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
}
#[test]
fn concurrent_duplicates_do_not_overwrite_each_other() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("a.txt");
    fs::write(&source, "data").unwrap();
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let source = source.clone();
            std::thread::spawn(move || duplicate(&source, "Copy").unwrap())
        })
        .collect();
    let copies: std::collections::BTreeSet<_> =
        threads.into_iter().map(|t| t.join().unwrap()).collect();
    assert_eq!(copies.len(), 4);
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 5);
    for copy in copies {
        assert_eq!(fs::read_to_string(copy).unwrap(), "data");
    }
}
#[cfg(unix)]
#[test]
fn failed_duplicate_leaves_source_intact_and_no_partial_copy() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("folder");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("file"), "original").unwrap();
    std::os::unix::fs::symlink(&source, source.join("cycle")).unwrap();
    assert!(duplicate(&source, "Copy").is_err());
    assert_eq!(fs::read_to_string(source.join("file")).unwrap(), "original");
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
}
