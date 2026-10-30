#!/usr/bin/env python3
"""Linux installer. Never runs Cargo as root or modifies shell startup files."""
import argparse
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shlex
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
import uuid

REPO = 'https://github.com/Bartzin55/Leditor.git'
APP = 'io.github.leditor.Leditor'
MARKER = 'leditor-installer-v1'


@contextmanager
def temporary_build_directory():
    temporary = tempfile.TemporaryDirectory(prefix='leditor-build-')
    try:
        yield temporary.name
    except (Exception, KeyboardInterrupt) as error:
        print('Installation stopped before cleanup:', error, file=sys.stderr, flush=True)
        raise
    finally:
        print('Removing temporary build files and tools. This may take several minutes.', flush=True)
        temporary.cleanup()
        print('Temporary build files removed.', flush=True)


def run(*args, **kwargs):
    return subprocess.run([str(a) for a in args], check=True, **kwargs)


def fetch(url, dest):
    print('Downloading:', url, flush=True)
    req = urllib.request.Request(url, headers={'User-Agent': 'Leditor-installer/1.5.1'})
    with urllib.request.urlopen(req, timeout=120) as response, open(dest, 'wb') as out:
        shutil.copyfileobj(response, out)


def extract(archive, dest):
    # Reject paths escaping the extraction directory, including archive symlinks.
    with tarfile.open(archive) as tar:
        for member in tar.getmembers():
            if member.issym() or member.islnk() or not (member.isfile() or member.isdir()):
                raise RuntimeError('Unexpected archive entry: ' + member.name)
            target = (dest / member.name).resolve()
            if dest.resolve() not in target.parents and target != dest.resolve():
                raise RuntimeError('Unsafe archive path')
        tar.extractall(dest)


def yes(prompt):
    return input(prompt + ' [y/N] ').strip().lower() in ('y', 'yes', 's', 'sim')


def native_available():
    if not all(shutil.which(x) for x in ('pkg-config', 'gcc', 'glib-compile-resources')):
        return False
    return subprocess.call(['pkg-config', '--atleast-version=4.10', 'gtk4']) == 0 and subprocess.call(
        ['pkg-config', '--atleast-version=5.0', 'gtksourceview-5']) == 0


def install(args):
    if platform.system() != 'Linux' or platform.machine() != 'x86_64' or platform.libc_ver()[0] != 'glibc':
        raise RuntimeError('Supported: Linux x86_64 with glibc. ARM and musl are not supported.')
    if os.geteuid() == 0:
        raise RuntimeError('Run as your normal user. System installation asks sudo only where needed.')
    scope = args.scope or ('system' if input('Install for [1] current user (default) or [2] system? ').strip() == '2' else 'user')
    admin = ['sudo'] if scope == 'system' else []
    if admin:
        run('sudo', '-v')
    home = Path.home()
    data = Path(os.environ.get('XDG_DATA_HOME', home / '.local/share'))
    base = Path('/usr/local/lib/leditor') if admin else data / 'leditor-install'
    bindir = Path('/usr/local/bin') if admin else home / '.local/bin'
    shared = Path('/usr/local/share') if admin else data
    if not base.is_absolute() or base.is_symlink() or any(c in str(base) + str(bindir) for c in '\n\r\t'):
        raise RuntimeError('Invalid installation directory')
    marker = base / '.leditor-managed'
    if base.exists() and (not marker.is_file() or marker.read_text().strip() != MARKER):
        raise RuntimeError('Refusing to overwrite an unmanaged directory: ' + str(base))
    command = bindir / 'leditor'
    desktop_path = shared / ('applications/' + APP + '.desktop')
    if desktop_path.exists() and 'X-Leditor-Managed=true' not in desktop_path.read_text():
        raise RuntimeError('An unmanaged desktop entry exists: ' + str(desktop_path))
    if os.path.lexists(command):
        old_target = os.readlink(command) if command.is_symlink() else ''
        allowed = (str(base / 'current/launch'), str(data / 'leditor/bin/leditor'))
        if old_target not in allowed:
            raise RuntimeError('An unmanaged command already exists: ' + str(command))
    with temporary_build_directory() as tmp:
        work = Path(tmp)
        env = os.environ.copy()
        # A private Cargo cache also keeps downloads separate from the user's files.
        env['CARGO_HOME'] = str(work / 'cargo')
        env['RUSTUP_HOME'] = str(work / 'rustup')
        env['CARGO_BUILD_JOBS'] = os.environ.get('CARGO_BUILD_JOBS', '2')
        mamba = None

        def conda(prefix, packages):
            nonlocal mamba
            if not mamba:
                mamba = shutil.which('micromamba')
                if not mamba:
                    archive = work / 'mamba.tar.bz2'
                    fetch('https://micro.mamba.pm/api/micromamba/linux-64/2.5.0', archive)
                    extract(archive, work)
                    mamba = str(work / 'bin/micromamba')
            run(mamba, '--no-rc', 'create', '-y', '-p', prefix, '-c', 'conda-forge',
                '--override-channels', *packages, env={**env, 'MAMBA_ROOT_PREFIX': str(work / 'mamba')})

        native = native_available()
        tools = []
        if not shutil.which('git'):
            tools.append('git')
        rust = shutil.which('rustc')
        cargo = shutil.which('cargo')
        if rust and cargo:
            version = run(rust, '--version', capture_output=True, text=True).stdout.split()[1]
            rust_ok = tuple(map(int, version.split('.')[:2])) >= (1, 88)
        else:
            rust_ok = False
        if rust_ok:
            # Resolve rustup proxies before isolating RUSTUP_HOME.
            if shutil.which('rustup'):
                rust = run('rustup', 'which', 'rustc', capture_output=True, text=True).stdout.strip()
                cargo = run('rustup', 'which', 'cargo', capture_output=True, text=True).stdout.strip()
            env['RUSTC'] = rust
        else:
            tools.append('rust>=1.88')
        if not native:
            tools += ['gcc_linux-64', 'pkg-config']
        if tools:
            conda(work / 'tools', tools)
            env['PATH'] = str(work / 'tools/bin') + os.pathsep + env['PATH']
        cargo = cargo if rust_ok else str(work / 'tools/bin/cargo')
        source = work / 'source'
        env['CARGO_TARGET_DIR'] = str(source / 'target')
        if args.source:
            original = Path(args.source).resolve()
            if not (original / 'Cargo.toml').is_file():
                raise RuntimeError('--source must contain Cargo.toml')
            shutil.copytree(original, source, ignore=shutil.ignore_patterns('.git', 'target', '__pycache__'))
            revision = 'local-' + hashlib.sha256((source / 'Cargo.lock').read_bytes() + (source / 'Cargo.toml').read_bytes()).hexdigest()[:12]
        else:
            run('git', 'clone', '--depth', '1', '--branch', args.ref, REPO, source, env=env)
            revision = run('git', '-C', source, 'rev-parse', 'HEAD', capture_output=True, text=True, env=env).stdout.strip()
        current = base / 'current'
        if not args.source and (current / 'revision').is_file() and (current / 'revision').read_text().strip() == revision:
            print('This revision is already installed.')
            return
        run(*admin, 'mkdir', '-p', base / 'releases')
        if not marker.exists():
            staged_marker = work / 'marker'
            staged_marker.write_text(MARKER + '\n')
            run(*admin, 'install', '-m644', staged_marker, marker)
        lock = base / '.install-lock'
        run(*admin, 'mkdir', lock)
        generation = base / 'releases' / uuid.uuid4().hex
        run(*admin, 'mkdir', generation)
        if admin:
            run('sudo', 'chown', str(os.getuid()) + ':' + str(os.getgid()), generation)
        committed = False
        try:
            runtime = generation / 'runtime'
            if not native:
                print('Preparing private GTK/GtkSourceView runtime (no system packages).', flush=True)
                conda(runtime, ['gtk4>=4.10', 'gtksourceview>=5,<6', 'glib', 'zlib', 'libffi', 'expat', 'libxml2-devel'])
                env['PKG_CONFIG_PATH'] = str(runtime / 'lib/pkgconfig') + ':' + str(runtime / 'share/pkgconfig')
                env['LD_LIBRARY_PATH'] = str(runtime / 'lib')
                env['PATH'] = str(runtime / 'bin') + ':' + env['PATH']
                env['CONDA_PREFIX'] = str(runtime)
                env['CC'] = str(work / 'tools/bin/x86_64-conda-linux-gnu-cc')
                env['CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER'] = env['CC']
                env['CARGO_ENCODED_RUSTFLAGS'] = '-L\x1fnative=' + str(runtime / 'lib')
            print('Building Leditor; this may take several minutes.', flush=True)
            run(cargo, 'build', '--locked', '--release', cwd=source, env=env)
            run(cargo, 'test', '--locked', '--lib', '--test', 'document', '--test', 'file_ops', cwd=source, env=env)
            print('Tests passed. Preparing application files and dependencies...', flush=True)
            shutil.copy2(source / 'target/release/leditor', generation / 'leditor')
            licenses = generation / 'LICENSES'
            shutil.copytree(source / 'LICENSES', licenses)
            sys.path.insert(0, str(source / 'scripts'))
            from collect_licenses import collect
            collect(source, licenses / 'rust', cargo, env)
            if not native:
                for package in (work / 'mamba/pkgs').glob('*/info/licenses'):
                    shutil.copytree(package, licenses / 'native' / package.parent.parent.name, dirs_exist_ok=True)
                shutil.copytree(runtime / 'conda-meta', licenses / 'native-metadata')
            pdfium = work / 'pdfium.tgz'
            fetch('https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7881/pdfium-linux-x64.tgz', pdfium)
            if hashlib.sha256(pdfium.read_bytes()).hexdigest() != '1470e21b8b4a3b4ad7f85684e2da11d94f3b69a86d81dee11b9b6709d927ac1d':
                raise RuntimeError('PDFium checksum mismatch')
            pdfdir = work / 'pdfium'
            pdfdir.mkdir()
            extract(pdfium, pdfdir)
            shutil.copy2(pdfdir / 'lib/libpdfium.so', generation / 'libpdfium.so')
            for path in pdfdir.iterdir():
                if path.name.lower().startswith(('license', 'notice')):
                    target = licenses / 'pdfium' / path.name
                    target.parent.mkdir(parents=True, exist_ok=True)
                    # The source bundle already contains PDFium notices. Merge the
                    # downloaded runtime's notices into this new generation only.
                    if path.is_dir():
                        shutil.copytree(path, target, dirs_exist_ok=True)
                    else:
                        shutil.copy2(path, target)
            launcher = '#!/bin/sh\n# Leditor managed launcher\nexport LEDITOR_MANAGED_INSTALL=1\n'
            if not native:
                launcher += 'export LD_LIBRARY_PATH=' + shlex.quote(str(runtime / 'lib')) + '\n'
                launcher += 'export XDG_DATA_DIRS=' + shlex.quote(str(runtime / 'share')) + ':"${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"\n'
                launcher += 'export GSETTINGS_SCHEMA_DIR=' + shlex.quote(str(runtime / 'share/glib-2.0/schemas')) + '\n'
            launcher += 'exec ' + shlex.quote(str(generation / 'leditor')) + ' "$@"\n'
            (generation / 'launch').write_text(launcher)
            (generation / 'launch').chmod(0o755)
            (generation / 'revision').write_text(revision + '\n')
            shutil.copy2(source / 'uninstall.sh', generation / 'uninstall.sh')
            result = run('ldd', generation / 'leditor', capture_output=True, text=True, env=env)
            if 'not found' in result.stdout:
                raise RuntimeError(result.stdout)
            result = run('ldd', generation / 'libpdfium.so', capture_output=True, text=True, env=env)
            if 'not found' in result.stdout:
                raise RuntimeError(result.stdout)
            old = os.readlink(current) if current.is_symlink() else None
            if admin:
                run('sudo', 'chown', '-R', 'root:root', generation)
            # Files are published only after a successful build and loader check.
            run(*admin, 'mkdir', '-p', bindir, shared / 'applications', shared / 'icons/hicolor/512x512/apps')
            run(*admin, 'ln', '-sfn', base / 'current/launch', command)
            desktop = work / 'leditor.desktop'
            quoted = str(command).replace('\\', '\\\\').replace('"', '\\"').replace('%', '%%').replace('$', '\\$').replace('`', '\\`')
            desktop.write_text('[Desktop Entry]\nType=Application\nName=Leditor\nExec="' + quoted.replace('\\', '\\\\') + '"\nIcon=' + APP + '\nTerminal=false\nCategories=Utility;TextEditor;\nX-Leditor-Managed=true\n')
            run(*admin, 'install', '-m644', desktop, shared / ('applications/' + APP + '.desktop'))
            run(*admin, 'install', '-m644', source / 'assets/leditor.png', shared / ('icons/hicolor/512x512/apps/' + APP + '.png'))
            run(*admin, 'ln', '-s', generation, base / 'current.next')
            run(*admin, 'mv', '-Tf', base / 'current.next', current)
            committed = True
            # Old generations intentionally retained for recovery; uninstaller removes them.
            if old:
                print('Previous generation retained for recovery:', old)
            if not yes('Remove the downloaded/copied source code?'):
                saved = home / ('leditor-source-' + uuid.uuid4().hex[:8])
                shutil.copytree(source, saved, ignore=shutil.ignore_patterns('target'))
                print('Source saved:', saved)
            print('Installed:', command)
            print('Uninstall: bash', str(base / 'current/uninstall.sh'), '--scope', scope)
            if str(bindir) not in os.environ.get('PATH', '').split(':'):
                print('Add this directory to PATH:', bindir)
        finally:
            if not committed:
                run(*admin, 'rm', '-rf', '--', generation)
            run(*admin, 'rmdir', lock)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Build and install Leditor without modifying existing toolchains.')
    parser.add_argument('--scope', choices=('user', 'system'))
    parser.add_argument('--ref', default='main', help='Git branch/tag to clone (default: main)')
    parser.add_argument('--source', help='Build a copy of this local source directory instead of cloning')
    try:
        install(parser.parse_args())
    except (Exception, KeyboardInterrupt) as error:
        print('Installation stopped:', error, file=sys.stderr)
        sys.exit(1)
