"""Installer safety tests; no network, administrator requests or real installation."""
import importlib.util
import io
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
import types
import argparse
from contextlib import redirect_stdout, redirect_stderr
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('installer', ROOT / 'scripts/install_linux.py')
installer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(installer)


class ArchiveTests(unittest.TestCase):
    def check_archive(self, name, link=False):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            archive = root / 'test.tar'
            with tarfile.open(archive, 'w') as tar:
                entry = tarfile.TarInfo(name)
                if link:
                    entry.type = tarfile.SYMTYPE
                    entry.linkname = '/etc/passwd'
                tar.addfile(entry, io.BytesIO())
            dest = root / 'out'
            dest.mkdir()
            installer.extract(archive, dest)
            return (dest / name).is_file()

    def test_normal_archive(self):
        self.assertTrue(self.check_archive('LICENSE'))

    def test_traversal_rejected(self):
        with self.assertRaises(RuntimeError):
            self.check_archive('../escape')

    def test_absolute_rejected(self):
        with self.assertRaises(RuntimeError):
            self.check_archive('/tmp/escape')

    def test_symlink_rejected(self):
        with self.assertRaises(RuntimeError):
            self.check_archive('escape', link=True)

    def test_native_detection_requires_tools(self):
        with patch.object(installer.shutil, 'which', return_value=None):
            self.assertFalse(installer.native_available())


class UninstallTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.home = Path(self.tmp.name)
        self.data = self.home / '.local/share'
        self.base = self.data / 'leditor-install'
        self.base.mkdir(parents=True)
        (self.base / '.leditor-managed').write_text(installer.MARKER)
        self.config = self.home / '.config/leditor'
        self.config.mkdir(parents=True)
        (self.config / 'settings.ini').write_text('keep')
        (self.data / 'leditor').mkdir()
        (self.data / 'leditor/session.ini').write_text('UNSAVED TEXT')
        self.bin = self.home / '.local/bin'
        self.bin.mkdir(parents=True)
        (self.bin / 'leditor').symlink_to(self.base / 'current/launch')
        self.env = {**os.environ, 'HOME': str(self.home), 'XDG_DATA_HOME': str(self.data),
                    'XDG_CONFIG_HOME': str(self.home / '.config'), 'XDG_CACHE_HOME': str(self.home / '.cache')}

    def tearDown(self):
        self.tmp.cleanup()

    def uninstall(self, answer):
        return subprocess.run(['bash', str(ROOT / 'uninstall.sh'), '--scope', 'user'],
                              env=self.env, input=answer, text=True, capture_output=True)

    def test_cancel_keeps_everything(self):
        self.assertEqual(self.uninstall('NO\n').returncode, 0)
        self.assertTrue(self.base.exists())

    def test_preserve_recovery_by_default(self):
        result = self.uninstall('REMOVE\nn\n')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(self.base.exists())
        self.assertTrue((self.data / 'leditor/session.ini').exists())
        self.assertTrue(self.config.exists())

    def test_explicit_data_removal(self):
        self.assertEqual(self.uninstall('REMOVE\ny\n').returncode, 0)
        self.assertFalse(self.config.exists())
        self.assertFalse((self.data / 'leditor').exists())

    def test_unknown_command_preserved(self):
        (self.bin / 'leditor').unlink()
        (self.bin / 'leditor').write_text('unrelated')
        self.assertEqual(self.uninstall('REMOVE\nn\n').returncode, 0)
        self.assertEqual((self.bin / 'leditor').read_text(), 'unrelated')

    def test_invalid_marker_rejected(self):
        (self.base / '.leditor-managed').write_text('other program')
        self.assertNotEqual(self.uninstall('REMOVE\ny\n').returncode, 0)
        self.assertTrue(self.base.exists())

    def test_active_installation_rejected(self):
        (self.base / '.install-lock').mkdir()
        self.assertNotEqual(self.uninstall('REMOVE\ny\n').returncode, 0)
        self.assertTrue(self.base.exists())


class InstallationFlowTests(unittest.TestCase):
    def exercise(self, fail=False):
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            data = home / 'data'
            base = data / 'leditor-install'
            base.mkdir(parents=True)
            (base / '.leditor-managed').write_text(installer.MARKER)
            marker_before = (base / '.leditor-managed').stat().st_mtime_ns
            old = base / 'releases/old'
            old.mkdir(parents=True)
            (old / 'leditor').write_text('old working binary')
            (base / 'current').symlink_to(old)
            source = home / 'original'
            (source / 'assets').mkdir(parents=True)
            bundled_notices = source / 'LICENSES/pdfium/licenses'
            bundled_notices.mkdir(parents=True)
            (bundled_notices / 'component.txt').write_bytes(b'bundled notice')
            (bundled_notices / 'bundled-only.txt').write_bytes(b'preserve notice')
            (source / 'Cargo.toml').write_text('version = "1.5.1"')
            (source / 'Cargo.lock').write_text('test lock')
            (source / 'assets/leditor.png').write_bytes(b'icon')
            (source / 'uninstall.sh').write_text('test')
            calls = []

            def fake_run(*args, **kwargs):
                args = [str(a) for a in args]
                calls.append(args)
                self.assertNotIn('sudo', args)
                if args[0] == '/test/rustc':
                    return subprocess.CompletedProcess(args, 0, 'rustc 1.99.1\n')
                if args[0] == '/test/cargo':
                    if fail:
                        raise subprocess.CalledProcessError(1, args)
                    binary = Path(kwargs['cwd']) / 'target/release/leditor'
                    binary.parent.mkdir(parents=True, exist_ok=True)
                    binary.write_text('new binary')
                    return subprocess.CompletedProcess(args, 0)
                if args[0] == 'ldd':
                    return subprocess.CompletedProcess(args, 0, 'resolved')
                return subprocess.run(args, check=True, **kwargs)

            def fake_fetch(url, dest):
                with tarfile.open(dest, 'w:gz') as tar:
                    for name, content in (
                        ('lib/libpdfium.so', b'pdfium'), ('LICENSE', b'PDFium license'),
                        ('licenses/component.txt', b'archive notice'),
                        ('licenses/nested/component.txt', b'nested notice'),
                    ):
                        entry = tarfile.TarInfo(name)
                        entry.size = len(content)
                        tar.addfile(entry, io.BytesIO(content))

            class Hash:
                def hexdigest(self):
                    return '1470e21b8b4a3b4ad7f85684e2da11d94f3b69a86d81dee11b9b6709d927ac1d'

            env = {**os.environ, 'HOME': str(home), 'XDG_DATA_HOME': str(data)}
            args = argparse.Namespace(scope='user', source=str(source), ref='main')
            with patch.dict(os.environ, env), patch.object(installer.os, 'geteuid', return_value=1000), \
                 patch.object(installer, 'native_available', return_value=True), \
                 patch.object(installer.shutil, 'which', side_effect=lambda x: None if x == 'rustup' else '/test/' + x), \
                 patch.object(installer, 'run', side_effect=fake_run), patch.object(installer, 'fetch', side_effect=fake_fetch), \
                 patch.object(installer.hashlib, 'sha256', return_value=Hash()), patch.object(installer, 'yes', return_value=True), \
                 patch.dict('sys.modules', {'collect_licenses': types.SimpleNamespace(collect=lambda *a: None)}):
                if fail:
                    with self.assertRaises(subprocess.CalledProcessError):
                        installer.install(args)
                    self.assertEqual((base / 'current').resolve(), old)
                    self.assertEqual(list((base / 'releases').iterdir()), [old])
                else:
                    installer.install(args)
                    self.assertNotEqual((base / 'current').resolve(), old)
                    self.assertIn('LEDITOR_MANAGED_INSTALL=1', (base / 'current/launch').read_text())
                    self.assertTrue((home / '.local/bin/leditor').is_symlink())
                    notices = base / 'current/LICENSES/pdfium'
                    self.assertEqual((notices / 'licenses/component.txt').read_bytes(), b'archive notice')
                    self.assertEqual((notices / 'licenses/bundled-only.txt').read_bytes(), b'preserve notice')
                    self.assertEqual((notices / 'licenses/nested/component.txt').read_bytes(), b'nested notice')
                    self.assertEqual((notices / 'LICENSE').read_bytes(), b'PDFium license')
                    self.assertEqual((bundled_notices / 'component.txt').read_bytes(), b'bundled notice')
                self.assertTrue((source / 'Cargo.toml').exists())
                self.assertFalse((base / '.install-lock').exists())
                self.assertTrue((old / 'leditor').exists())
                self.assertEqual((base / '.leditor-managed').stat().st_mtime_ns, marker_before)
                self.assertEqual((base / '.leditor-managed').read_text(), installer.MARKER)

    def test_user_update_no_sudo_preserves_original_source(self):
        self.exercise()

    def test_failed_build_keeps_old_installation(self):
        self.exercise(fail=True)


class CleanupTests(unittest.TestCase):
    def exercise(self, fail=False):
        output, errors = io.StringIO(), io.StringIO()
        with tempfile.TemporaryDirectory() as outside:
            existing_tool = Path(outside) / 'user-rustc'
            existing_tool.write_text('keep')
            failure = RuntimeError('simulated build failure')
            with redirect_stdout(output), redirect_stderr(errors):
                try:
                    with installer.temporary_build_directory() as tmp:
                        work = Path(tmp)
                        (work / 'target').mkdir()
                        (work / 'target/binary').write_text('temporary')
                        (work / 'external-tool').symlink_to(existing_tool)
                        if fail:
                            raise failure
                except RuntimeError as error:
                    self.assertIs(error, failure)
            self.assertFalse(work.exists())
            self.assertEqual(existing_tool.read_text(), 'keep')
            self.assertIn('Removing temporary build files', output.getvalue())
            self.assertIn('Temporary build files removed.', output.getvalue())
            if fail:
                self.assertIn(str(failure), errors.getvalue())

    def test_success_cleanup_preserves_external_tool(self):
        self.exercise()

    def test_failure_cleanup_preserves_error_and_external_tool(self):
        self.exercise(fail=True)


class WindowsBundleTests(unittest.TestCase):
    def test_recursive_dlls_and_missing_dependency(self):
        spec = importlib.util.spec_from_file_location('bundle', ROOT / 'scripts/bundle_windows.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            prefix, output = root / 'ucrt64', root / 'app'
            (prefix / 'bin').mkdir(parents=True)
            output.mkdir()
            for name in ('a.dll', 'b.dll', 'unused-compiler.dll'):
                (prefix / 'bin' / name).write_text(name)
            (output / 'leditor.exe').write_text('exe')
            def dependencies(args, **kwargs):
                return {'leditor.exe': 'DLL Name: a.dll', 'a.dll': 'DLL Name: b.dll',
                        'b.dll': 'DLL Name: a.dll'}[Path(args[-1]).name]
            with patch.object(module.subprocess, 'check_output', side_effect=dependencies):
                module.bundle(prefix, output)
            self.assertTrue((output / 'b.dll').exists())
            self.assertFalse((output / 'unused-compiler.dll').exists())
            with patch.dict(os.environ, {'SystemRoot': str(root / 'windows')}), \
                 patch.object(module.subprocess, 'check_output', return_value='DLL Name: absent.dll'):
                with self.assertRaises(RuntimeError):
                    module.bundle(prefix, output)


if __name__ == '__main__':
    unittest.main()
