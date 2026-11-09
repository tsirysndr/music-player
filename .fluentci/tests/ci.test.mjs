import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { setupEnvironment, persistEnvironment } from '../github-env.mjs';
import * as ci from '../ci/output/Main/index.js';
import * as support from '../ci/output/CI.Support/index.js';
import * as packaging from '../ci/output/CI.Package/index.js';
import * as duckdb from '../ci/output/CI.DuckDB/index.js';

function fixture(env = {}, listing = '', contents = {}) {
  const calls = [];
  let command = [];
  const unquote = token => token.slice(1, -1).replaceAll("'\\''", "'");
  globalThis.Memory = { fromString: value => ({ offset: value }), find: value => ({ readString: () => value }) };
  globalThis.Host = { outputString() {}, getFunctions: () => new Proxy({}, { get: (_, name) => value => {
    calls.push([name, value]);
    if (name === 'pipeline' || name === 'file' || name === 'as_service') return JSON.stringify({ id: 'fixture' });
    if (name === 'get_os') return 'linux';
    if (name === 'get_arch') return 'x86_64';
    if (name === 'has_env') return JSON.stringify(Object.hasOwn(env, value));
    if (name === 'get_env') return env[value] || '';
    if (name === 'with_exec') command = JSON.parse(value).map(unquote);
    if (name === 'sha256') return 'f'.repeat(64);
    if (name === 'stdout') {
      if (command[0] === 'pwd') return '/checkout';
      if (command[0] === 'mktemp') return '/tmp/fixture';
      if (command[0] === 'find') return listing;
      if (command[0] === 'cat') return contents[command[1]] ?? 'done';
      return 'done';
    }
    if (name === 'execute_plan') return JSON.stringify({ outcome: 'succeeded', tasks: [{ id: 'write', status: 'succeeded', exit_code: 0, error: null, duration_ms: 0, stdout: '/tmp/fixture/stdout.log', stderr: '/tmp/fixture/stderr.log', cache: { outcome: 'disabled', key: null, reason: '' }, wait_reason: null }] });
    return '';
  } }) };
  return calls;
}
const commands = calls => calls.filter(([name]) => name === 'with_exec').map(([, value]) => JSON.parse(value).map(token => token.slice(1, -1).replaceAll("'\\''", "'")));

test('setup results persist PATH and Rust settings after logs and multiline JSON', () => {
  const environment = [{ name: 'PATH', value: '/tools/bin:/usr/bin' }, { name: 'RUSTUP_HOME', value: '/tools/rustup' }];
  const output = 'installer logs\n' + JSON.stringify({ environment }, null, 2);
  assert.deepEqual(setupEnvironment(output), environment);
  const file = join(mkdtempSync(join(tmpdir(), 'fluentci-env-')), 'env');
  persistEnvironment(output, file);
  assert.match(readFileSync(file, 'utf8'), /RUSTUP_HOME<</);
  assert.match(readFileSync(file, 'utf8'), /\/tools\/bin:\/usr\/bin/);
  assert.throws(() => setupEnvironment('installer failed'), /native environment/);
  assert.throws(() => setupEnvironment(JSON.stringify({ environment: [{ name: 'BAD\nKEY', value: 'x' }] })), /Invalid/);
});

test('web checks retain typechecking, coverage and both builds without shell commands', () => {
  const calls = fixture(); ci.webCheck();
  const args = commands(calls);
  for (const expected of [['bunx', 'tsc', '--noEmit'], ['bun', 'run', 'test:coverage'], ['bunx', 'vite', 'build'], ['bunx', 'storybook', 'build']]) {
    assert.ok(args.some(command => JSON.stringify(command) === JSON.stringify(expected)));
  }
  assert.ok(args.every(command => !['bash', 'sh'].includes(command[0])));
});

test('release validation happens before build/file effects', () => {
  const calls = fixture();
  assert.throws(() => ci.release({ source: '.', target: 'x86_64-unknown-linux-gnu', version: '../bad', desktop: true })(), /Invalid release/);
  assert.equal(calls.length, 0);
  assert.throws(() => ci.release({ source: '.', target: 'aarch64-unknown-linux-gnu', version: 'v1.0.0', desktop: false })(), /Target does not match/);
  assert.equal(commands(calls).length, 0);
});

test('release packaging selects native targets, preserves names and uses native file writes', () => {
  const calls = fixture();
  ci.release({ source: '.', target: 'x86_64-unknown-linux-gnu', version: 'v1.2.3', desktop: false })();
  const args = commands(calls);
  assert.ok(args.some(c => c[0] === 'cargo' && c.includes('--locked')));
  assert.ok(args.some(c => c[0] === 'tar' && c.includes('/checkout/artifacts/release/music-player_v1.2.3_x86_64-unknown-linux-gnu.tar.gz')));
  assert.ok(calls.some(([name]) => name === 'execute_plan'));
  assert.ok(args.every(c => !c.includes('-c')));
});

test('invalid Nix attributes fail before host execution', () => {
  const calls = fixture();
  assert.throws(() => ci.nixPackage({ attribute: '--help' })(), /Unknown Nix/);
  assert.equal(calls.length, 0);
});

test('coverage commit and flags reach Codecov as literal argv', () => {
  const calls = fixture(); ci.codecov({ report: 'coverage/lcov.info', commit: 'abc123', flag: 'webui' })();
  assert.deepEqual(commands(calls)[0], ['codecovcli', 'upload-process', '--fail-on-error', '--disable-search', '-f', 'coverage/lcov.info', '-C', 'abc123', '-F', 'webui']);
});

test('environment expansion is literal, single-pass and supports braces', () => {
  fixture({ TARGET: 'value with spaces; $(touch bad) $UNCHANGED', RUN: '42' });
  assert.equal(support.expand('prefix-${TARGET}-$RUN')(), 'prefix-value with spaces; $(touch bad) $UNCHANGED-42');
});

test('GitHub asset upload rejects empty directories and passes filenames literally', () => {
  fixture(); assert.throws(() => ci.githubAssets({ directory: 'dist', tag: 'v1' })(), /No release assets/);
  const calls = fixture({}, 'dist/file with spaces.tar.gz\ndist/archive.sha256\ndist/debian/DEBIAN/control\ndist/package.spec\n');
  ci.githubAssets({ directory: 'dist', tag: 'v1' })();
  assert.deepEqual(commands(calls).at(-1), ['gh', 'release', 'upload', 'v1', 'dist/file with spaces.tar.gz', 'dist/archive.sha256', '--clobber']);
});


test('macOS packaging retains app identity, signing, DMG name and clean archives', () => {
  const calls = fixture();
  packaging.mac('/bin')('v1.2.3')('aarch64-apple-darwin')('desktop.tar.gz')('/out')();
  const args = commands(calls);
  assert.ok(args.some(c => c[0] === 'codesign' && c.includes('com.github.tsirysndr.music-player')));
  assert.ok(args.some(c => c[0] === 'hdiutil' && c.includes('/out/Music_Player_v1.2.3_aarch64.dmg')));
  assert.ok(args.some(c => c[0] === 'env' && c.includes('COPYFILE_DISABLE=1')));
  const writes = calls.filter(([name]) => name === 'execute_plan').map(([, value]) => JSON.parse(value).plan.tasks[0].file_operation);
  assert.ok(writes.some(op => op?.text?.includes('<key>CFBundleVersion</key><string>1.2.3</string>')));
  assert.ok(args.every(c => !['bash', 'sh'].includes(c[0])));
});

test('DuckDB staging reuses only a complete matching archive', () => {
  const calls = fixture({}, './vendor/duckdb/lib/.stamp\n./vendor/duckdb/lib/libduckdb_static.a\n');
  // The fixture returns a different stamp, so this must fetch and replace it.
  duckdb.fetch('.')('x86_64-unknown-linux-gnu')();
  const args = commands(calls);
  assert.ok(args.some(c => c[0] === 'curl' && c.at(-1).endsWith('static-libs-linux-amd64.zip')));
  assert.ok(args.every(c => !['bash', 'sh'].includes(c[0])));
  const writes = calls.filter(([name]) => name === 'execute_plan').map(([, value]) => JSON.parse(value).plan.tasks[0].file_operation);
  assert.ok(writes.some(op => op?.text === '1.5.5-linux-amd64'));
});


test('a matching complete DuckDB stamp skips network and file writes', () => {
  const calls = fixture({}, './vendor/duckdb/lib/.stamp\n./vendor/duckdb/lib/libduckdb_static.a\n', { './vendor/duckdb/lib/.stamp': '1.5.5-linux-amd64' });
  duckdb.fetch('.')('x86_64-unknown-linux-gnu')();
  assert.ok(commands(calls).every(c => c[0] !== 'curl'));
  assert.ok(calls.every(([name]) => name !== 'execute_plan'));
});

test('Gemfury keeps credentials out of argv and skips either missing secret', () => {
  for (const env of [{}, { FURY_TOKEN: 'test-secret' }, { FURY_ACCOUNT: 'account' }]) {
    const calls = fixture(env); ci.gemfury({ files: ['dist/app.deb'] })();
    assert.equal(commands(calls).length, 0);
  }
  const calls = fixture({ FURY_TOKEN: 'test-secret', FURY_ACCOUNT: 'account' });
  ci.gemfury({ files: ['dist/file with spaces.deb'] })();
  const upload = commands(calls).find(c => c[0] === 'curl');
  assert.ok(upload.includes('{{FURY_TOKEN}}:'));
  assert.ok(upload.includes('package=@"dist/file with spaces.deb"'));
  assert.ok(!JSON.stringify(calls).includes('test-secret'));
});


test('integration retains CLI lookup and database settings around its managed service', () => {
  const calls = fixture({ PATH: '/usr/bin' });
  ci.integration({ target: 'x86_64-unknown-linux-gnu' })();
  const settings = calls.filter(([name]) => name === 'set_envs').map(([, value]) => value).join('\n');
  assert.ok(settings.includes('/checkout/target/debug:/usr/bin'));
  assert.ok(settings.includes('DATABASE_URL'));
  assert.ok(settings.includes('sqlite:///tmp/music-player.sqlite3'));
  assert.ok(calls.some(([name]) => name === 'as_service'));
  assert.ok(commands(calls).every(c => !['bash', 'sh'].includes(c[0])));
});
