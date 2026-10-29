#!/usr/bin/env python3
"""Maintainer utility: fetch locked crates, verify Cargo checksums, retain legal files.

Requires Python 3.11+. No crate code is executed. Used to prepare source releases.
"""
import concurrent.futures
import hashlib
import io
import json
import os
from pathlib import Path
import tarfile
import tomllib
import urllib.request
import urllib.error


def package_licenses(package, destination):
    name, version = package['name'], package['version']
    url = f'https://static.crates.io/crates/{name}/{name}-{version}.crate'
    cache = Path(os.environ.get('CARGO_HOME', Path.home() / '.cargo'))
    archives = list(cache.glob(f'registry/cache/*/{name}-{version}.crate'))
    if archives:
        data = archives[0].read_bytes()
    else:
        with urllib.request.urlopen(url, timeout=120) as response:
            data = response.read()
    if hashlib.sha256(data).hexdigest() != package['checksum']:
        raise RuntimeError('Checksum mismatch: ' + name)
    output = destination / (name + '-' + version)
    output.mkdir(parents=True, exist_ok=True)
    record = {'name': name, 'version': version, 'sha256': package['checksum'], 'source': url}
    count = 0
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        for member in archive.getmembers():
            rel = Path(*Path(member.name).parts[1:])
            if member.isfile() and any(rel.name.lower().startswith(x) for x in ('license', 'licence', 'copying', 'copyright', 'notice', 'authors')):
                if '..' in rel.parts or rel.is_absolute():
                    raise RuntimeError('Unsafe crate path')
                target = output / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(archive.extractfile(member).read())
                count += 1
        if not count:
            # Some crates omit workspace-root licenses from the published archive.
            manifest = tomllib.loads(archive.extractfile(f'{name}-{version}/Cargo.toml').read().decode())
            vcs = json.load(archive.extractfile(f'{name}-{version}/.cargo_vcs_info.json'))
            repo = manifest['package']['repository'].removesuffix('.git').rstrip('/')
            if not repo.startswith(('https://github.com/', 'https://gitlab.gnome.org/')):
                raise RuntimeError('Manual license review needed: ' + name)
            sha = vcs['git']['sha1']
            raw = repo.replace('github.com', 'raw.githubusercontent.com') + '/' + sha if repo.startswith('https://github.com/') else repo + '/-/raw/' + sha
            filenames = ('LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE', 'LICENSE.md', 'LICENSE.txt', 'AUTHORS')
            subdir = vcs.get('path_in_vcs', '')
            dirs = [subdir] + [str(p) for p in Path(subdir).parents if str(p) != '.'] if subdir else []
            candidates = list(filenames) + [d + '/' + name for d in dirs for name in filenames]
            for filename in candidates:
                try:
                    with urllib.request.urlopen(raw + '/' + filename, timeout=60) as response:
                        (output / Path(filename).name).write_bytes(response.read())
                        count += 1
                except urllib.error.HTTPError as error:
                    if error.code != 404:
                        raise
            if not count:
                if not repo.startswith('https://github.com/'):
                    raise RuntimeError('No license file: ' + name)
                req = urllib.request.Request(repo.replace('https://github.com/', 'https://api.github.com/repos/') + '/git/trees/' + sha + '?recursive=1', headers={'User-Agent': 'Leditor-license-collector'})
                with urllib.request.urlopen(req, timeout=60) as response:
                    tree = json.load(response)
                for item in tree['tree']:
                    rel = Path(item['path'])
                    if item['type'] == 'blob' and any(rel.name.lower().startswith(x) for x in ('license', 'licence', 'copying', 'copyright', 'notice')):
                        with urllib.request.urlopen(repo.replace('github.com', 'raw.githubusercontent.com') + '/' + sha + '/' + item['path'], timeout=60) as response:
                            target = output / rel
                            target.parent.mkdir(parents=True, exist_ok=True)
                            target.write_bytes(response.read())
                            count += 1
                if not count:
                    raise RuntimeError('No license file: ' + name)
    print(name, version, flush=True)
    return record


if __name__ == '__main__':
    root = Path(__file__).resolve().parent.parent
    packages = [p for p in tomllib.loads((root / 'Cargo.lock').read_text())['package'] if p.get('source', '').startswith('registry+')]
    dest = root / 'LICENSES/rust'
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        rows = list(pool.map(lambda p: package_licenses(p, dest), packages))
    (dest / 'packages.json').write_text(json.dumps(rows, indent=2) + '\n')
