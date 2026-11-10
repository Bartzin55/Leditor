//! Conservative filesystem operations: never replace an existing destination,
//! never follow directory symlinks, and stage recursive copies before publishing.
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
    // A preflight existence check alone is insufficient: another process can
    // create the destination between that check and rename. No copy fallback
    // or replacement is permitted, including cross-volume moves.
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{renameat_with, RenameFlags, CWD};
        renameat_with(CWD, source, CWD, target, RenameFlags::NOREPLACE).map_err(Into::into)
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::MoveFileExW;
        let wide = |path: &Path| -> io::Result<Vec<u16>> {
            let mut text: Vec<_> = path.as_os_str().encode_wide().collect();
            if text.contains(&0) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "NUL in path"));
            }
            text.push(0);
            Ok(text)
        };
        let source = wide(source)?;
        let target = wide(target)?;
        // SAFETY: both buffers are live, NUL-terminated UTF-16. Flags=0 refuses
        // existing destinations and cross-volume copy/delete fallbacks.
        if unsafe { MoveFileExW(source.as_ptr(), target.as_ptr(), 0) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
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
/// Copy beside the original, preserving the extension and never replacing data.
/// Stage once, then retry publication if another process claims the same name.
pub fn duplicate(source: &Path, copy_label: &str) -> io::Result<PathBuf> {
    let parent = source
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = source
        .file_name()
        .ok_or_else(|| io::Error::other("Missing filename"))?;
    let meta = fs::symlink_metadata(source)?;
    let extension = if meta.is_file() {
        source.extension()
    } else {
        None
    };
    let stem = if extension.is_some() {
        source.file_stem().unwrap_or(name)
    } else {
        name
    };
    let staging = tempfile::Builder::new()
        .prefix(".leditor-copy-")
        .tempdir_in(parent)?;
    let staged = staging.path().join("entry");
    copy_tree(source, &staged)?;
    for number in 1..=u32::MAX {
        let mut name = stem.to_os_string();
        name.push(format!(" ({copy_label} {number})"));
        if let Some(extension) = extension {
            name.push(".");
            name.push(extension);
        }
        let target = parent.join(name);
        match move_entry(&staged, &target) {
            Ok(()) => return Ok(target),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::other("No available copy name"))
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
