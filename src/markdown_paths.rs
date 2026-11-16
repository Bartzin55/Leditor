//! Markdown paths are relative to the document or rooted in the open project.
use gtk::{gio, glib, prelude::*};
use std::path::{Path, PathBuf};

pub fn project_root(source: Option<&Path>, project: Option<&Path>) -> Option<PathBuf> {
    project
        .or_else(|| source.and_then(Path::parent))?
        .canonicalize()
        .ok()
}

pub fn resolve(url: &str, source: Option<&Path>, project: Option<&Path>) -> Option<PathBuf> {
    if url.chars().any(char::is_control) {
        return None;
    }
    let root = project_root(source, project)?;
    if glib::uri_parse_scheme(url).is_some_and(|s| s.eq_ignore_ascii_case("file")) {
        let path = gio::File::for_uri(url).path()?.canonicalize().ok()?;
        return (path.starts_with(&root) && path.is_file()).then_some(path);
    }
    if glib::uri_parse_scheme(url).is_some() {
        return None;
    }
    let decoded = glib::uri_unescape_string(url, None::<&str>)?;
    let decoded = decoded.replace('\\', "/");
    // No UNC shares or drive letters: a leading slash means this project.
    if decoded.starts_with("//") || decoded.contains(':') || decoded.contains('\0') {
        return None;
    }
    let mut path = if decoded.starts_with('/') {
        root.clone()
    } else {
        source?.parent()?.canonicalize().ok()?
    };
    if !path.starts_with(&root) {
        return None;
    }
    for part in decoded.split('/') {
        match part {
            "" | "." => (),
            ".." => {
                if path == root || !path.pop() {
                    return None;
                }
            }
            part => path.push(part),
        }
    }
    let path = path.canonicalize().ok()?;
    (path.starts_with(&root) && path.is_file()).then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn project_paths_are_bounded_and_decoded() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("project");
        std::fs::create_dir_all(root.join("docs")).unwrap();
        std::fs::create_dir(root.join("assets")).unwrap();
        let image = root.join("assets/my image.png");
        std::fs::write(&image, []).unwrap();
        let source = root.join("docs/readme.md");
        for url in [
            "/assets/my%20image.png",
            "../assets/my%20image.png",
            "\\assets\\my%20image.png",
        ] {
            assert_eq!(
                resolve(url, Some(&source), Some(&root)),
                Some(image.clone())
            );
        }
        for url in [
            "../../secret.png",
            "/../secret.png",
            "%2e%2e/%2e%2e/secret.png",
            "//server/share.png",
            "C:\\secret.png",
        ] {
            assert!(resolve(url, Some(&source), Some(&root)).is_none(), "{url}");
        }
        assert!(resolve("../assets/my%20image.png", Some(&source), None).is_none());
        #[cfg(unix)]
        {
            let outside = temp.path().join("secret.png");
            std::fs::write(&outside, []).unwrap();
            std::os::unix::fs::symlink(&outside, root.join("escape.png")).unwrap();
            assert!(resolve("/escape.png", Some(&source), Some(&root)).is_none());
        }
    }
}
