//! Conservative filesystem operations: never replace an existing destination,
//! never follow directory symlinks, and stage recursive copies before publishing.
use gtk::{gio, prelude::*};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn child(parent: &Path, name: &str) -> io::Result<PathBuf> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.ends_with([' ', '.'])
        || name
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid filename",
        ));
    }
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    if ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Reserved filename",
        ));
    }
    Ok(fs::canonicalize(parent)?.join(name))
}
fn error(e: gtk::glib::Error) -> io::Error {
    io::Error::other(e.to_string())
}
fn destination(source: &Path, target: &Path) -> io::Result<()> {
    if fs::symlink_metadata(target).is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Destination already exists",
        ));
    }
    let source = fs::canonicalize(source)?;
    let parent = fs::canonicalize(
        target
            .parent()
            .ok_or_else(|| io::Error::other("Missing parent directory"))?,
    )?;
    if parent.starts_with(&source) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot place a folder inside itself",
        ));
    }
    Ok(())
}
pub fn move_entry(source: &Path, target: &Path) -> io::Result<()> {
    destination(source, target)?;
    // NO_FALLBACK_FOR_MOVE rejects cross-volume moves instead of risking a
    // partially completed recursive copy/delete. COPY_NONE never overwrites.
    gio::File::for_path(source)
        .move_(
            &gio::File::for_path(target),
            gio::FileCopyFlags::NOFOLLOW_SYMLINKS | gio::FileCopyFlags::NO_FALLBACK_FOR_MOVE,
            gio::Cancellable::NONE,
            None,
        )
        .map_err(error)
}
fn copy_tree(source: &Path, target: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(source)?;
    if meta.file_type().is_symlink() {
        return Err(io::Error::other("Copying symbolic links is not supported"));
    }
    if meta.is_dir() {
        fs::create_dir(target)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_tree(&entry.path(), &target.join(entry.file_name()))?;
        }
    } else if meta.is_file() {
        let mut input = fs::File::open(source)?;
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)?;
        io::copy(&mut input, &mut output)?;
        output.sync_all()?;
    } else {
        return Err(io::Error::other("Unsupported filesystem entry"));
    }
    fs::set_permissions(target, meta.permissions())?;
    Ok(())
}
pub fn copy_entry(source: &Path, target: &Path) -> io::Result<()> {
    destination(source, target)?;
    let staging = tempfile::Builder::new()
        .prefix(".leditor-copy-")
        .tempdir_in(target.parent().unwrap())?;
    let staged = staging.path().join("entry");
    copy_tree(source, &staged)?;
    move_entry(&staged, target)
}
pub fn create(parent: &Path, name: &str, folder: bool) -> io::Result<PathBuf> {
    let target = child(parent, name)?;
    if folder {
        fs::create_dir(&target)?;
    } else {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)?
            .sync_all()?;
    }
    Ok(target)
}
pub fn delete(path: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if meta.is_dir() && !meta.file_type().is_symlink() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}
/// Match descendants before a move/delete, while the old path still exists.
pub fn relative(path: &Path, ancestor: &Path) -> Option<PathBuf> {
    if let Ok(relative) = path.strip_prefix(ancestor) {
        return Some(relative.to_owned());
    }
    if ancestor.is_symlink() {
        return None;
    }
    fs::canonicalize(path)
        .ok()?
        .strip_prefix(fs::canonicalize(ancestor).ok()?)
        .ok()
        .map(Path::to_owned)
}
