"""Copy the PE dependency closure, including GdkPixbuf's dynamically loaded plugins."""
from pathlib import Path
import os
import re
import shutil
import subprocess
import sys


def bundle(prefix, output):
    prefix, output = Path(prefix), Path(output)
    available = {p.name.lower(): p for p in (prefix / 'bin').glob('*.dll')}
    pending = [output / 'leditor.exe']
    plugins = prefix / 'lib/gdk-pixbuf-2.0'
    if plugins.exists():
        shutil.copytree(plugins, output / 'gdk-pixbuf-2.0')
        pending += list((output / 'gdk-pixbuf-2.0').rglob('*.dll'))
    query = prefix / 'bin/gdk-pixbuf-query-loaders.exe'
    if query.exists():
        shutil.copy2(query, output / query.name)
        pending.append(output / query.name)
    seen = set()
    while pending:
        file = pending.pop()
        text = subprocess.check_output([str(prefix / 'bin/objdump.exe'), '-p', str(file)], text=True)
        for name in re.findall(r'DLL Name:\s*(\S+)', text):
            lower = name.lower()
            if lower in seen:
                continue
            seen.add(lower)
            if lower in available:
                target = output / available[lower].name
                shutil.copy2(available[lower], target)
                pending.append(target)
            elif not lower.startswith(('api-ms-', 'ext-ms-')) and not (Path(os.environ['SystemRoot']) / 'System32' / name).exists():
                raise RuntimeError('Unresolved runtime DLL: ' + name)
    for name in ('glib-2.0', 'gtksourceview-5', 'icons', 'locale', 'themes', 'fonts', 'mime'):
        source = prefix / 'share' / name
        if source.exists():
            shutil.copytree(source, output / 'share' / name)


if __name__ == '__main__':
    bundle(*sys.argv[1:])
