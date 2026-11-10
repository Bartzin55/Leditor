# Dependency licenses — Leditor 1.8.0-alpha

See LICENSES/ for collected texts and provenance. Native PDFium notices are
included; installers also preserve exact GTK runtime package notices. The
scripts use micromamba/conda-forge on Linux and MSYS2/UCRT64 on Windows; these are
installation tools, not new linked Rust application dependencies.

Leditor's source remains MIT. Its license does not replace licenses on linked libraries or bundled data. This source archive does not redistribute native GTK/PDFium binaries.

| Component | License | Role |
|---|---|---|
| Comrak 0.59.0 | BSD-2-Clause | Markdown parsing; no CLI or Syntect features |
| emojis / finl_unicode / unicode-normalization | MIT or Apache-2.0 plus Unicode data terms (see exact notices) | Emoji and Unicode processing |
| rustix | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | Atomic no-replace filesystem operation on Linux |
| sourceview5 Rust bindings | MIT | GtkSourceView interface |
| GtkSourceView native library | LGPL-2.1-or-later | Editing, minimap, gutter, indentation, case conversion and syntax highlighting |
| gtk-rs Rust bindings | MIT | GTK/GLib/GIO bindings |
| GTK native libraries | LGPL family; see component notices | Native GUI |
| pdfium-render | MIT OR Apache-2.0 (MIT selected) | PDFium Rust bindings |
| PDFium | BSD-style; additional third-party notices | PDF renderer |
| GdkPixbuf | LGPL-2.1-or-later; loaders have their own notices | PNG/JPEG decode |
| fluent-bundle / unic-langid | MIT OR Apache-2.0 | Existing localization |
| tempfile | MIT OR Apache-2.0 | Existing atomic file operations |
| windows-sys | MIT OR Apache-2.0 | Windows theme query and atomic no-replace moves |

GtkSourceView does not require changing Leditor's own license to GPL. Distributors must still comply with LGPL requirements for the native libraries (including notices, applicable source availability and replacement/relinking rights). Do not describe LGPL libraries themselves as MIT. Prefer normal dynamic linking and retain the runtime's licenses. Modifying LGPL components brings obligations for those modifications.

PDFium's own BSD-style license is permissive, but its compiled distribution includes dependencies; retain the complete LICENSE and licenses/ directory from the chosen build. Do not replace PDFium with a GPL/AGPL renderer without a separate license review.

Primary references:
- https://gitlab.gnome.org/World/Rust/sourceview5-rs
- https://gitlab.gnome.org/GNOME/gtksourceview
- https://github.com/ajrcarey/pdfium-render/blob/master/LICENSE.md
- https://pdfium.googlesource.com/pdfium/+/refs/heads/main/LICENSE
- https://github.com/bblanchon/pdfium-binaries

Cargo.lock records exact Rust dependency versions. Before distributing a complete Windows/Linux binary bundle, review the actual native components and codecs included in that bundle; this source ZIP is not an audit of an unbuilt binary distribution.

The added Comrak dependency graph uses permissive BSD/MIT/Apache/Zlib and Unicode
licenses. Comrak's COPYING also describes CC-BY-SA licensing of its CommonMark
specification test document; that document is not embedded into Leditor. Preserve
all collected notices. Cargo.lock pins the exact versions; `LICENSES/rust/packages.json`
records archive checksums. These additions do not require relicensing Leditor.
