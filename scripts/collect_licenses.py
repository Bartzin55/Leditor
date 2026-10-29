"""Collect exact Cargo package license/copyright files, including transitive crates."""
import json
import os
from pathlib import Path
import shutil
import subprocess


def collect(source, destination, cargo='cargo', env=None):
    destination = Path(destination)
    destination.mkdir(parents=True, exist_ok=True)
    metadata = json.loads(subprocess.check_output(
        [str(cargo), 'metadata', '--locked', '--format-version', '1'], cwd=source, env=env))
    manifest = []
    for package in metadata['packages']:
        root = Path(package['manifest_path']).parent
        if not package['source']:
            continue
        output = destination / (package['name'] + '-' + package['version'])
        output.mkdir(exist_ok=True)
        copied = []
        for path in root.rglob('*'):
            if path.is_file() and any(path.name.lower().startswith(x) for x in ('license', 'licence', 'copying', 'copyright', 'notice', 'authors')):
                rel = path.relative_to(root)
                (output / rel).parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(path, output / rel)
                copied.append(str(rel))
        if package.get('license_file'):
            path = root / package['license_file']
            shutil.copy2(path, output / path.name)
            copied.append(path.name)
        if not copied:
            bundled = Path(source) / 'LICENSES/rust' / output.name
            if bundled.is_dir() and any(bundled.iterdir()):
                shutil.copytree(bundled, output, dirs_exist_ok=True)
            else:
                raise RuntimeError('License text missing for ' + package['name'] + '; refusing incomplete installation')
        manifest.append({k: package.get(k) for k in ('name', 'version', 'license', 'repository', 'source')})
    (destination / 'packages.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')


if __name__ == '__main__':
    import sys
    collect(Path.cwd(), sys.argv[1], os.environ.get('CARGO', 'cargo'))
