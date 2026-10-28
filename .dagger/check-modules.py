#!/usr/bin/env python3
"""Compile every standalone release entrypoint against the checked-in SDK.

Each module references the shared SDK and library. Stage only those inputs to
verify that no undeclared files from the repository are needed.
"""
from pathlib import Path
import shutil
import subprocess
import tempfile

root = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix='dagger-release-check-') as tmp:
    stage = Path(tmp)
    shutil.copytree(root / 'lib', stage / 'lib')
    shutil.copytree(root / 'ci/dagger-sdk', stage / 'ci/dagger-sdk')
    for module in sorted((root / 'modules').iterdir()):
        if not (module / 'dagger-module.toml').exists():
            continue
        dest = stage / 'modules' / module.name
        shutil.copytree(module, dest, ignore=shutil.ignore_patterns('output', '.spago', 'node_modules', 'dagger-sdk'))
        subprocess.run(['spago', 'build', '--pedantic-packages', '--output', str(stage / 'output')], cwd=dest, check=True)
        shutil.copyfile(dest / 'spago.lock', module / 'spago.lock')
        print(f'Checked {module.name}', flush=True)
