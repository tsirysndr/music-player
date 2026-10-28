#!/usr/bin/env python3
"""Check module and workflow wiring without Dagger."""
from pathlib import Path
import json
import re
import tomllib
import yaml
root = Path(__file__).resolve().parent.parent
# Standalone modules must retain their shared inputs and workspace scopes.
workspace = tomllib.loads((root / 'dagger.toml').read_text())
expected = {
    'linux-packaging': {'deb', 'rpm'},
    'archive': {'tarGz', 'sha256'},
    'gemfury': {'upload'},
    'github-assets': {'upload'},
}
for name, exports in expected.items():
    module = root / '.dagger/modules' / name
    config = tomllib.loads((module / 'dagger-module.toml').read_text())
    assert config['name'] == name, module
    assert config['runtime']['source'] == '../../sdk/purescript', module
    assert {'../../lib/**', '../../ci/dagger-sdk/**'} <= set(config['include']), module
    assert workspace['sdks']['purescript']['scopes'][f'.dagger/modules/{name}']['name'] == name
    actual = set(re.findall(r'Module.function "([^"]+)"', (module / 'src/Main.purs').read_text()))
    assert actual == exports, (module, actual)
    lock = json.loads((module / 'spago.lock').read_text())
    assert 'release-' + name in lock['workspace']['packages'], module
print(f'Checked {len(expected)} standalone release modules and their shared source wiring.')

ci_functions = set(re.findall(r'Module.function "([^"]+)"', (root / '.dagger/ci/src/Main.purs').read_text()))
cli_functions = {re.sub(r'[A-Z]', lambda m: '-' + m[0].lower(), name) for name in ci_functions}
for path in (root / '.github/workflows').glob('*.yml'):
    doc = yaml.safe_load(path.read_text())
    for job in doc['jobs'].values():
        steps = job.get('steps', [])
        for index, step in enumerate(steps):
            calls = re.findall(r'dagger(?: --progress (?:plain|logs))?(?: -v)? api call -m (\.dagger/[^ ]+) ([a-z][a-z0-9-]*)', step.get('run', ''))
            for module, function in calls:
                functions = cli_functions if module == '.dagger/ci' else {re.sub(r'[A-Z]', lambda m: '-' + m[0].lower(), name) for name in expected[module.rsplit('/', 1)[1]]}
                assert function in functions, (path, function)
                assert any(s.get('uses') == './.github/actions/setup-dagger' for s in steps[:index]), path
print('Checked CI and release workflow module calls; no engine started.')
