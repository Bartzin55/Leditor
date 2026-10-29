# Leditor and third-party notices

Leditor is MIT; see `Leditor-MIT.txt`. Third-party components retain their own
licenses and copyrights. These notices do not relicense those components as MIT.

`rust/` contains legal files from the exact archives named in Cargo.lock, including
transitive and target-specific dependencies. The maintainer collection utility
checks each archive against Cargo.lock. `rust/packages.json` records provenance.
Where an archive omits its workspace license, the utility uses the repository
commit from `.cargo_vcs_info.json`. r-efi carries its MIT permission text and
copyrights in AUTHORS. MIT is selected where a dependency offers MIT as an option.

Installers also retain the LICENSE and licenses directory from PDFium Chromium
7881, and legal texts/metadata from the native packages actually installed.
System GTK packages remain managed by the distribution, with their notices in
the distribution's documentation directories. The private Linux runtime includes
conda package metadata; the Windows bundle retains MSYS2 share/licenses.
`native/` also includes notices from the GTK-related conda-forge packages used
during this release's validation; actual installed versions may differ.

GTK/GtkSourceView and some dependencies use LGPL licenses. Leditor links these
libraries dynamically; replacing them must remain possible. Redistribution of
a built installation requires fulfilling the applicable source-availability and
relinking/replacement obligations, not merely including license texts. This ZIP
contains source code and notices, not a ready-to-redistribute native binary bundle.
The installer downloads native binaries from their original distributors for
local use. Preserve matching source/build recipes when publishing binary releases.

Tool licenses (Rust, GCC, MSYS2, micromamba) do not change the license of Leditor's
source. Temporary tools are not included in the installed application. Compiler
runtime libraries keep their own licenses and applicable runtime exceptions.
