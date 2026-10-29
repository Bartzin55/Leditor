//! Per-user Linux integration; never invokes sudo or changes shell startup files.
use std::{
    fs,
    io::{self, Read, Write},
    os::unix::fs::{symlink, PermissionsExt},
    path::Path,
};

fn same_content(a: &Path, b: &Path) -> io::Result<bool> {
    if !b.exists() {
        return Ok(false);
    }
    if fs::metadata(a)?.len() != fs::metadata(b)?.len() {
        return Ok(false);
    }
    let mut a = fs::File::open(a)?;
    let mut b = fs::File::open(b)?;
    let (mut left, mut right) = ([0; 65536], [0; 65536]);
    loop {
        let n = a.read(&mut left)?;
        b.read_exact(&mut right[..n])?;
        if left[..n] != right[..n] {
            return Ok(false);
        }
        if n == 0 {
            return Ok(true);
        }
    }
}
fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(());
    }
    let mut tmp = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
    tmp.write_all(bytes)?;
    tmp.as_file().sync_all()?;
    tmp.persist(path).map_err(|e| e.error)?;
    Ok(())
}
fn desktop_exec(path: &Path) -> io::Result<String> {
    let text = path
        .to_str()
        .ok_or_else(|| io::Error::other("Non-UTF-8 installation path"))?;
    if text.contains(['\n', '\r', '\0']) {
        return Err(io::Error::other("Invalid installation path"));
    }
    // Desktop Entry escaping: value escaping is applied after Exec quoting.
    let mut quoted = String::from("\"");
    for ch in text.chars() {
        match ch {
            '%' => quoted.push_str("%%"),
            '\\' | '"' | '`' | '$' => {
                quoted.push('\\');
                quoted.push(ch);
            }
            _ => quoted.push(ch),
        }
    }
    quoted.push('"');
    Ok(quoted.replace('\\', "\\\\").replace('\t', "\\t"))
}
pub fn install(source: &Path, data: &Path, home: &Path) -> io::Result<()> {
    let app_dir = data.join("leditor/bin");
    let bin_dir = home.join(".local/bin");
    let applications = data.join("applications");
    let executable = app_dir.join("leditor");
    let command = bin_dir.join("leditor");
    // Do not replace an unrelated executable or user-created launcher.
    if fs::symlink_metadata(&command).is_ok()
        && fs::read_link(&command).ok().as_deref() != Some(executable.as_path())
    {
        return Err(io::Error::other(format!(
            "{} already exists; integration skipped",
            command.display()
        )));
    }
    let desktop = applications.join("io.github.leditor.Leditor.desktop");
    if desktop.exists() && !fs::read_to_string(&desktop)?.contains("X-Leditor-Managed=true\n") {
        return Err(io::Error::other(
            "An unmanaged Leditor desktop entry already exists",
        ));
    }
    let entry = format!("[Desktop Entry]\nType=Application\nName=Leditor\nComment=Text editor\nExec={}\nIcon=io.github.leditor.Leditor\nTerminal=false\nCategories=Utility;TextEditor;\nStartupNotify=true\nX-Leditor-Managed=true\n", desktop_exec(&executable)?);
    fs::create_dir_all(&app_dir)?;
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&applications)?;
    if source != executable && !same_content(source, &executable)? {
        let mut tmp = tempfile::NamedTempFile::new_in(&app_dir)?;
        io::copy(&mut fs::File::open(source)?, &mut tmp)?;
        tmp.as_file()
            .set_permissions(fs::Permissions::from_mode(0o755))?;
        tmp.as_file().sync_all()?;
        tmp.persist(&executable).map_err(|e| e.error)?;
    }
    if fs::symlink_metadata(&command).is_err() {
        symlink(&executable, &command)?;
    }
    let icons = data.join("icons/hicolor/512x512/apps");
    fs::create_dir_all(&icons)?;
    atomic_write(
        &icons.join("io.github.leditor.Leditor.png"),
        include_bytes!("../assets/leditor.png"),
    )?;
    atomic_write(&desktop, entry.as_bytes())?;
    Ok(())
}
pub fn integrate() -> io::Result<()> {
    // Script-managed installations need their runtime launcher, not a bare EXE copy.
    if std::env::var_os("LEDITOR_MANAGED_INSTALL").is_some() {
        return Ok(());
    }
    let home = gtk::glib::home_dir();
    // A system installation is managed by its administrator/package manager.
    let source = std::env::current_exe()?;
    if source.starts_with("/usr") || source.starts_with("/bin") || source.starts_with("/opt") {
        return Ok(());
    }
    install(&source, &gtk::glib::user_data_dir(), &home)?;
    let bin = home.join(".local/bin");
    if !std::env::var_os("PATH")
        .is_some_and(|value| std::env::split_paths(&value).any(|p| p == bin))
    {
        eprintln!(
            "Leditor: add {} to PATH to use the leditor command in this shell.",
            bin.display()
        );
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn install_update_and_conflict() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home with spaces");
        let data = home.join(".local/share");
        let source = temp.path().join("build");
        fs::write(&source, b"first binary").unwrap();
        install(&source, &data, &home).unwrap();
        assert_eq!(
            fs::read(data.join("icons/hicolor/512x512/apps/io.github.leditor.Leditor.png"))
                .unwrap(),
            include_bytes!("../assets/leditor.png")
        );
        assert!(
            fs::read_to_string(data.join("applications/io.github.leditor.Leditor.desktop"))
                .unwrap()
                .contains("Icon=io.github.leditor.Leditor")
        );
        install(&source, &data, &home).unwrap();
        assert_eq!(
            fs::read(home.join(".local/bin/leditor")).unwrap(),
            b"first binary"
        );
        fs::write(&source, b"updated binary").unwrap();
        install(&source, &data, &home).unwrap();
        assert_eq!(
            fs::read(home.join(".local/bin/leditor")).unwrap(),
            b"updated binary"
        );
        fs::remove_file(home.join(".local/bin/leditor")).unwrap();
        fs::write(home.join(".local/bin/leditor"), b"unrelated").unwrap();
        assert!(install(&source, &data, &home).is_err());
        assert_eq!(
            fs::read(home.join(".local/bin/leditor")).unwrap(),
            b"unrelated"
        );
    }
    #[test]
    fn desktop_path_quoting() {
        assert_eq!(
            desktop_exec(Path::new("/tmp/a b%/leditor")).unwrap(),
            "\"/tmp/a b%%/leditor\""
        );
        assert!(desktop_exec(Path::new("/tmp/line\nbreak")).is_err());
    }
}
