#!/usr/bin/env bash
# Bootstrap only. All downloaded tools stay in a private temporary directory.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
if [[ ${1:-} == --help ]]; then
    echo 'Usage: bash install.sh [--scope user|system] [--source DIR] [--ref REF]'
    exit 0
fi
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || { echo 'Only Linux x86_64 is supported.' >&2; exit 1; }
command -v getconf >/dev/null && getconf GNU_LIBC_VERSION >/dev/null || { echo 'A glibc Linux distribution is required.' >&2; exit 1; }
if command -v python3 >/dev/null && python3 -c 'import sys; assert sys.version_info >= (3,9)' 2>/dev/null; then
    exec python3 scripts/install_linux.py "$@"
fi
# Choose scope before downloading the Python bootstrap as well.
has_scope=false
for arg in "$@"; do [[ $arg != --scope && $arg != --scope=* ]] || has_scope=true; done
if ! $has_scope; then
    read -r -p 'Install for [1] current user (default) or [2] system? ' answer
    scope=user; [[ $answer != 2 ]] || scope=system
    set -- --scope "$scope" "$@"
fi
for tool in curl tar bzip2; do command -v "$tool" >/dev/null || { echo "Bootstrap requires $tool." >&2; exit 1; }; done
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
curl -fL --retry 3 https://micro.mamba.pm/api/micromamba/linux-64/2.5.0 -o "$work/mamba.tar.bz2"
tar -xjf "$work/mamba.tar.bz2" -C "$work" bin/micromamba
MAMBA_ROOT_PREFIX="$work/cache" "$work/bin/micromamba" --no-rc create -y -p "$work/python" -c conda-forge --override-channels python=3.12
"$work/python/bin/python" scripts/install_linux.py "$@"
