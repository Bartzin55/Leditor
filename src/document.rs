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
#[derive(Debug, Default)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub text: String,
    saved_text: String,
    bom: bool,
    crlf: bool,
}

impl Document {
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
