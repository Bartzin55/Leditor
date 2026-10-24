use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

/// Structured read failures keep user-visible text in the localization layer.
#[derive(Debug)]
pub enum OpenError {
    Io(io::Error),
    Encoding,
    Binary,
}
impl From<io::Error> for OpenError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
impl std::fmt::Display for OpenError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Encoding => formatter.write_str("Unsupported text encoding"),
            Self::Binary => formatter.write_str("NUL bytes in text file"),
        }
    }
}
impl std::error::Error for OpenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// A single UTF-8 document. `saved_text` is the last successfully persisted snapshot.
#[derive(Debug, Default, Clone)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub text: String,
    saved_text: String,
    bom: bool,
    crlf: bool,
}

impl Document {
    pub fn autosave(&mut self) -> io::Result<()> {
        let path = self
            .path
            .clone()
            .ok_or_else(|| io::Error::other("Untitled document"))?;
        let disk = Self::open(&path).map_err(|e| io::Error::other(e.to_string()))?;
        if disk.saved_text != self.saved_text || disk.bom != self.bom || disk.crlf != self.crlf {
            return Err(io::Error::other(format!(
                "{} changed outside Leditor; use Save or Save As after review",
                path.display()
            )));
        }
        self.save_to(&path)
    }
    pub fn accept_saved_snapshot(&mut self, saved: &Self) {
        self.saved_text.clone_from(&saved.saved_text);
    }
    pub fn saved_content(&self) -> &str {
        &self.saved_text
    }
    pub fn write_recovery(&self, key: &gtk::glib::KeyFile, group: &str) {
        use gtk::prelude::*;
        if let Some(path) = &self.path {
            key.set_string(group, "path", &gtk::gio::File::for_path(path).uri());
        }
        key.set_string(group, "text", &self.text);
        // A detached empty document uses a sentinel which KeyFile cannot store.
        key.set_string(
            group,
            "saved",
            if self.saved_text == "\0" {
                ""
            } else {
                &self.saved_text
            },
        );
        key.set_boolean(group, "dirty", self.is_dirty());
        key.set_boolean(group, "bom", self.bom);
        key.set_boolean(group, "crlf", self.crlf);
    }
    pub fn from_recovery(key: &gtk::glib::KeyFile, group: &str) -> Result<Self, gtk::glib::Error> {
        use gtk::prelude::*;
        let text = key.string(group, "text")?.to_string();
        let mut saved_text = key.string(group, "saved")?.to_string();
        if key.boolean(group, "dirty")? && text == saved_text {
            saved_text.push('\0');
        }
        Ok(Self {
            path: key
                .string(group, "path")
                .ok()
                .and_then(|s| gtk::gio::File::for_uri(&s).path()),
            text,
            saved_text,
            bom: key.boolean(group, "bom")?,
            crlf: key.boolean(group, "crlf")?,
        })
    }

    pub fn open(path: &Path) -> Result<Self, OpenError> {
        let bytes = fs::read(path)?;
        let bom = bytes.starts_with(&[0xef, 0xbb, 0xbf]);
        let payload = if bom { &bytes[3..] } else { &bytes[..] };
        let text = std::str::from_utf8(payload).map_err(|_| OpenError::Encoding)?;
        if text.contains('\0') {
            return Err(OpenError::Binary);
        }
        // Normalize uniform Windows newlines for editing, then restore them on save.
        // Mixed line endings are retained verbatim instead of silently rewriting them.
        let crlf = text.contains("\r\n") && !text.replace("\r\n", "").contains(['\r', '\n']);
        let text = if crlf {
            text.replace("\r\n", "\n")
        } else {
            text.to_owned()
        };
        Ok(Self {
            path: Some(path.to_owned()),
            saved_text: text.clone(),
            text,
            bom,
            crlf,
        })
    }

    /// Detach after deletion: retain all text and force the next save to Save As.
    pub fn detach(&mut self) {
        self.path = None;
        self.saved_text.clear();
        // Even an empty saved file must remain a document the user can save.
        if self.text.is_empty() {
            self.saved_text.push('\0');
        }
    }
    pub fn is_dirty(&self) -> bool {
        self.text != self.saved_text
    }

    /// Writes a sibling temporary file, flushes it, then atomically replaces the target.
    /// State is committed only after success. Existing symlinks retain their identity.
    pub fn save_to(&mut self, path: &Path) -> io::Result<()> {
        let target = if path.is_symlink() {
            fs::canonicalize(path)?
        } else {
            path.to_owned()
        };
        let parent = target
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        if let Ok(metadata) = fs::metadata(&target) {
            temporary
                .as_file()
                .set_permissions(metadata.permissions())?;
        }
        if self.bom {
            temporary.write_all(&[0xef, 0xbb, 0xbf])?;
        }
        if self.crlf {
            temporary.write_all(self.text.replace('\n', "\r\n").as_bytes())?;
        } else {
            temporary.write_all(self.text.as_bytes())?;
        }
        temporary.as_file().sync_all()?;
        temporary.persist(&target).map_err(|error| error.error)?;
        self.path = Some(path.to_owned());
        self.saved_text.clone_from(&self.text);
        Ok(())
    }
}

/// Canonical paths also collapse relative paths and existing symbolic links.
pub fn same_path(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}
