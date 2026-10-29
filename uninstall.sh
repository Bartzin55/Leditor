#!/usr/bin/env bash
set -euo pipefail
scope=user
if [[ ${1:-} == --scope && ${2:-} =~ ^(user|system)$ ]]; then scope=$2
elif [[ $# != 0 ]]; then echo 'Usage: bash uninstall.sh [--scope user|system]'; exit 1
else read -r -p 'Remove [1] user (default) or [2] system installation? ' answer; [[ $answer != 2 ]] || scope=system
fi
data=${XDG_DATA_HOME:-$HOME/.local/share}
config=${XDG_CONFIG_HOME:-$HOME/.config}
cache=${XDG_CACHE_HOME:-$HOME/.cache}
base="$data/leditor-install"; bin="$HOME/.local/bin"; shared="$data"; admin=()
if [[ $scope == system ]]; then base=/usr/local/lib/leditor; bin=/usr/local/bin; shared=/usr/local/share; admin=(sudo); fi
[[ ! -L $base && -f $base/.leditor-managed && $(< "$base/.leditor-managed") == leditor-installer-v1 ]] || { echo 'No managed installation at:' "$base"; exit 1; }
[[ ! -d $base/.install-lock ]] || { echo 'An installation is in progress; removal cancelled.'; exit 1; }
echo "Remove Leditor from $base? Close Leditor before continuing."
read -r -p 'Type REMOVE to confirm: ' answer
[[ $answer == REMOVE ]] || exit 0
if [[ $scope == system ]]; then sudo -v; fi
if [[ -L $bin/leditor && $(readlink -- "$bin/leditor") == "$base/current/launch" ]]; then "${admin[@]}" rm -- "$bin/leditor"; fi
desktop="$shared/applications/io.github.leditor.Leditor.desktop"
if [[ -f $desktop ]] && grep -q '^X-Leditor-Managed=true$' "$desktop"; then "${admin[@]}" rm -- "$desktop"; fi
"${admin[@]}" rm -f -- "$shared/icons/hicolor/512x512/apps/io.github.leditor.Leditor.png"
"${admin[@]}" rm -rf -- "$base"
if [[ $scope == user && ! -L $data/leditor && ! -L $data/leditor/bin ]]; then
    # Remove the old 1.3–1.5 automatic binary copy, but retain session.ini.
    rm -f -- "$data/leditor/bin/leditor"
fi
read -r -p 'Remove YOUR settings and recovered/unsaved documents too? [y/N] ' answer
if [[ $answer =~ ^([yY]|[sS])$ ]]; then
    for parent in "$config" "$data" "$cache"; do
        [[ $parent == /* && $parent != / && ! -L $parent/leditor ]] || { echo 'Unsafe data path; skipped:' "$parent"; continue; }
        rm -rf -- "$parent/leditor"
    done
fi
echo 'Leditor removed. Other users, retained sources and shared system dependencies were not modified.'
