#!/usr/bin/env bash
# Leditor 1.7.0-alpha: existing GTK 4 / GtkSourceView 5 dependencies cover this release.
# Bootstrap only. All downloaded tools stay in a private temporary directory.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
if [[ ${1:-} == --help ]]; then
    echo 'Usage: bash install.sh [--scope user|system] [--source DIR] [--ref REF]'
    exit 0
fi
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || { echo 'Only Linux x86_64 is supported.' >&2; exit 1; }
command -v getconf >/dev/null && getconf GNU_LIBC_VERSION >/dev/null || { echo 'A glibc Linux distribution is required.' >&2; exit 1; }
has_python=false
if command -v python3 >/dev/null && python3 -c 'import sys; assert sys.version_info >= (3,9)' 2>/dev/null; then
    has_python=true
    if [[ -f scripts/install_linux.py ]]; then
        exec python3 scripts/install_linux.py "$@"
    fi
fi
# Choose scope before downloading the Python bootstrap as well.
has_scope=false
for arg in "$@"; do [[ $arg != --scope && $arg != --scope=* ]] || has_scope=true; done
if ! $has_scope; then
    read -r -p 'Install for [1] current user (default) or [2] system? ' answer
    scope=user; [[ $answer != 2 ]] || scope=system
    set -- --scope "$scope" "$@"
fi
command -v curl >/dev/null || { echo 'Bootstrap requires curl.' >&2; exit 1; }
work=$(mktemp -d -t leditor-bootstrap.XXXXXXXX)
cleanup_bootstrap() {
    echo 'Removing temporary Python/bootstrap tools. This may take several minutes.'
    if rm -rf -- "$work"; then
        echo 'Temporary bootstrap tools removed.'
    else
        echo "Could not completely remove temporary bootstrap tools: $work" >&2
    fi
}
trap cleanup_bootstrap EXIT
installer=scripts/install_linux.py
if [[ ! -f $installer ]]; then
    # A standalone install.sh fetches its helper; the helper downloads the
    # selected project revision. Keep the caller's local-source path unchanged.
    installer="$work/install_linux.py"
    curl -fL --retry 3 https://raw.githubusercontent.com/Bartzin55/Leditor/main/scripts/install_linux.py -o "$installer"
fi
if $has_python; then
    python3 "$installer" "$@"
    exit 0
fi
for tool in tar bzip2; do command -v "$tool" >/dev/null || { echo "Bootstrap requires $tool." >&2; exit 1; }; done
curl -fL --retry 3 https://micro.mamba.pm/api/micromamba/linux-64/2.5.0 -o "$work/mamba.tar.bz2"
tar -xjf "$work/mamba.tar.bz2" -C "$work" bin/micromamba
MAMBA_ROOT_PREFIX="$work/cache" "$work/bin/micromamba" --no-rc create -y -p "$work/python" -c conda-forge --override-channels python=3.12
"$work/python/bin/python" "$installer" "$@"
