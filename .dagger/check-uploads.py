"""Execute workflow upload shell blocks with a fake dagger executable only."""
from pathlib import Path
import json, os, subprocess, tempfile, yaml

def check(repo, names):
    tested = 0
    for name in names:
        doc = yaml.safe_load((repo / '.github/workflows' / name).read_text())
        for job in doc['jobs'].values():
            for step in job.get('steps', []):
                script = step.get('run', '')
                if 'api call -m .dagger/modules/' not in script:
                    continue
                if '/gemfury' in script:
                    assert "steps.gemfury.outputs.available == 'true'" in step.get('if', ''), name
                else:
                    assert '--clobber' in script, name
                with tempfile.TemporaryDirectory() as tmp:
                    cwd = Path(tmp)
                    (cwd/'bin').mkdir()
                    fake = cwd/'bin/dagger'
                    fake.write_text('#!/usr/bin/env python3\nimport json,os,sys\nopen(os.environ["CALL_LOG"],"a").write(json.dumps(sys.argv[1:])+"\\n")\n')
                    fake.chmod(0o755)
                    assets = cwd/'artifacts/release'
                    assets.mkdir(parents=True)
                    nested = name == 'release-slint-desktop.yml'
                    paths = ['archive with spaces.tar.gz', 'archive with spaces.tar.gz.sha256', ('dist/' if nested else '')+'example.deb', ('dist/' if nested else '')+'example.rpm', ('dist/' if nested else '')+'example.deb.sha256']
                    for path in paths:
                        p=assets/path; p.parent.mkdir(exist_ok=True); p.write_text('fixture')
                    env = dict(os.environ, PATH=str(cwd/'bin')+os.pathsep+os.environ['PATH'], CALL_LOG=str(cwd/'calls'), GITHUB_REPOSITORY='owner/repo', RELEASE_TAG='v1', RELEASE_VERSION='v1', GITHUB_RUN_ID='123', GITHUB_RUN_ATTEMPT='2', GITHUB_JOB='release', TARGET='aarch64-unknown-linux-gnu', FURY_ACCOUNT='test', FURY_TOKEN='unused', GH_TOKEN='unused')
                    script = script.replace('${{ matrix.target }}', 'aarch64-unknown-linux-gnu')
                    subprocess.run(['bash','-euo','pipefail','-c',script],cwd=cwd,env=env,check=True)
                    calls = [json.loads(line) for line in (cwd/'calls').read_text().splitlines()]
                    assert len(calls)==1, calls
                    args=calls[0]
                    selected = [args[i+1] for i,a in enumerate(args) if a=='--files']
                    expected = [p for p in paths if p.endswith(('.deb','.rpm'))] if '/gemfury' in script else paths
                    assert sorted(selected)==sorted(expected), (name, selected, expected)
                    assert '--run-id' in args and 'env:' in args[args.index('--token')+1]
                    for path in paths: (assets/path).unlink()
                    empty = subprocess.run(['bash','-euo','pipefail','-c',script],cwd=cwd,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
                    assert empty.returncode != 0
                    assert len((cwd/'calls').read_text().splitlines()) == 1
                    tested += 1
    expected_count = 6 if (repo / '.github/workflows/linux-x86_64-build.yml').exists() else 4
    assert tested == expected_count, (tested, expected_count)
    print(f'{repo.name}: {tested} workflow upload blocks verified with fake dagger; no engine or uploads.')

def check_codecov(repo):
    for name, report, flags in [('ci.yml', 'artifacts/lcov.info', []), ('webui-tests.yml', 'artifacts/webui/coverage/lcov.info', ['--flags', 'webui'])]:
        doc = yaml.safe_load((repo / '.github/workflows' / name).read_text())
        steps = [step for job in doc['jobs'].values() for step in job.get('steps', [])]
        assert not any('codecov/codecov-action' in step.get('uses', '') for step in steps)
        uploads = [step for step in steps if '.dagger/modules/codecov upload' in step.get('run', '')]
        assert len(uploads) == 1
        step = uploads[0]
        assert step['continue-on-error'] is True
        assert 'pull_request.head.sha' in step['env']['COVERAGE_COMMIT']
        assert 'github.head_ref' in step['env']['COVERAGE_BRANCH']
        assert 'pull_request.number' in step['env']['COVERAGE_PR']
        with tempfile.TemporaryDirectory() as tmp:
            cwd = Path(tmp)
            fake = cwd / 'dagger'
            fake.write_text('#!/usr/bin/env python3\nimport json,os,sys\nopen(os.environ["CALL_LOG"],"w").write(json.dumps(sys.argv[1:]))\n')
            fake.chmod(0o755)
            for pr in ['', '42']:
                env = dict(os.environ, PATH=str(cwd)+os.pathsep+os.environ['PATH'], CALL_LOG=str(cwd/'calls'), GITHUB_REPOSITORY='owner/repo', GITHUB_RUN_ID='123', GITHUB_RUN_ATTEMPT='2', GITHUB_JOB='tests', CODECOV_TOKEN='unused', COVERAGE_COMMIT='a'*40, COVERAGE_BRANCH='feature/with spaces', COVERAGE_PR=pr)
                subprocess.run(['bash', '-euo', 'pipefail', '-c', step['run']], cwd=cwd, env=env, check=True)
                args = json.loads((cwd/'calls').read_text())
                assert args == ['--progress', 'logs', '-v', 'api', 'call', '-m', '.dagger/modules/codecov', 'upload', '--report', report, '--repository', 'owner/repo', '--commit', 'a'*40, '--branch', 'feature/with spaces', '--pull-request', pr, '--token', 'env:CODECOV_TOKEN', '--run-id', '123-2-tests'] + flags, args
    print('Checked both Codecov workflow calls for push and PR metadata with a fake executable.')

if __name__ == '__main__':
    root=Path(__file__).resolve().parent.parent
    names = ['linux-x86_64-build.yml','linux-aarch64-build.yml','linux-armhf-build.yml'] if (root / '.github/workflows/linux-x86_64-build.yml').exists() else ['release.yml','release-for-arm.yml','release-slint-desktop.yml']
    check(root, names)
    check_codecov(root)
