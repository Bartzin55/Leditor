"""Standalone shell bootstrap: isolate downloads and execute only a fake helper."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class BootstrapTests(unittest.TestCase):
    def run_bootstrap(self, *, local=False, download_fails=False, helper_exit=0):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / 'download with spaces'
            source.mkdir()
            shutil.copy2(ROOT / 'install.sh', source / 'install.sh')
            tools = root / 'tools'
            tools.mkdir()
            temp = root / 'temporary'
            temp.mkdir()
            result_file = root / 'result.json'
            helper = root / 'helper.py'
            helper.write_text(
                'import json, os, sys\n'
                'from pathlib import Path\n'
                'Path(os.environ["BOOTSTRAP_RESULT"]).write_text(json.dumps('
                '{"argv": sys.argv[1:], "cwd": os.getcwd()}))\n'
                f'sys.exit({helper_exit})\n'
            )
            if local:
                (source / 'scripts').mkdir()
                shutil.copy2(helper, source / 'scripts/install_linux.py')
            curl = tools / 'curl'
            curl.write_text(
                '#!/bin/bash\n'
                'touch "$BOOTSTRAP_CURL_CALLED"\n'
                + ('exit 22\n' if download_fails else
                   'while [[ $# -gt 0 ]]; do\n'
                   '  if [[ $1 == -o ]]; then cp "$BOOTSTRAP_HELPER" "$2"; exit 0; fi\n'
                   '  shift\n'
                   'done\nexit 2\n')
            )
            curl.chmod(0o755)
            called = root / 'curl-called'
            env = dict(os.environ, PATH=str(tools) + os.pathsep + os.environ['PATH'],
                       TMPDIR=str(temp), BOOTSTRAP_HELPER=str(helper),
                       BOOTSTRAP_RESULT=str(result_file), BOOTSTRAP_CURL_CALLED=str(called))
            args = ['--scope', 'user', '--source', '/source with spaces', '--ref', 'main']
            result = subprocess.run(['bash', str(source / 'install.sh'), *args],
                                    cwd=root, env=env, capture_output=True, text=True)
            self.assertEqual(list(temp.iterdir()), [], result.stdout + result.stderr)
            self.assertEqual(called.exists(), not local)
            if download_fails and not local:
                self.assertEqual(result.returncode, 22)
                self.assertFalse(result_file.exists())
            else:
                self.assertEqual(result.returncode, helper_exit, result.stderr)
                self.assertEqual(json.loads(result_file.read_text()),
                                 {'argv': args, 'cwd': str(source)})

    def test_standalone_downloads_helper_and_forwards_arguments(self):
        self.run_bootstrap()

    def test_local_source_uses_bundled_helper_without_download(self):
        self.run_bootstrap(local=True)

    def test_failed_download_stops_and_cleans_temporary_files(self):
        self.run_bootstrap(download_fails=True)

    def test_helper_failure_propagates_and_cleans_temporary_files(self):
        self.run_bootstrap(helper_exit=7)
