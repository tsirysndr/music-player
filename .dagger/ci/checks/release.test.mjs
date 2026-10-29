import assert from 'node:assert/strict';
import test from 'node:test';
import { runAff } from '../output/Effect.Aff/index.js';
import { Left } from '../output/Data.Either/index.js';
import { connectWith } from '../output/Dagger/index.js';
import * as Q from '../output/Dagger.Query/index.js';
import * as C from '../output/Dagger.Container/index.js';
import * as Deb from '../output/Release.Deb/index.js';
import * as Rpm from '../output/Release.Rpm/index.js';
import * as Tar from '../output/Release.TarGz/index.js';
import * as Sha from '../output/Release.Sha256/index.js';
import * as Fury from '../output/Release.Gemfury/index.js';
import * as Github from '../output/Release.GithubAssets/index.js';
import * as Codecov from '../output/Release.Codecov/index.js';

const client = connectWith({ port: 1, token: 'test', timeoutMs: 1 })();
const dir = Q.directory(client);
const file = C.file('/fixture')(Q.container(client));
const token = Q.setSecret('test-token')('not-a-real-token')(client);
const run = aff => new Promise((resolve, reject) => runAff(result => () =>
  result instanceof Left ? reject(result.value0) : resolve(result.value0))(aff)());
const arg = (step, name) => run(step.args.find(arg => arg.name === name).value);
const commands = graph => Promise.all(graph.steps.filter(s => s.name === 'withExec').map(s => arg(s, 'args')));

test('generic packages have no Rockbox paths or metadata', async () => {
  const deb = await run(Deb.build(client)({root: dir, name: 'example 1.deb'}));
  assert.deepEqual((await commands(deb)).at(-1), ['dpkg-deb', '--build', '--root-owner-group', '/package', '/work/example 1.deb']);
  const rpm = Rpm.buildIn('/custom.spec')('aarch64')(Q.container(client));
  assert.deepEqual((await commands(rpm)).at(-1), ['rpmbuild', '--define', '_topdir /tmp/rpmbuild', '--target', 'aarch64', '-bb', '/custom.spec']);
});

test('archive and checksum treat spaces and option-like names literally', async () => {
  const tar = await run(Tar.create(client)({source: dir, name: 'example.tar.gz', members: ['-file', 'with spaces']}));
  assert.deepEqual((await commands(tar)).at(-1), ['tar', '-czf', '/work/example.tar.gz', '-C', '/input', '--', '-file', 'with spaces']);
  const sha = await run(Sha.generate(client)({file, name: '-example.tar.gz'}));
  assert.deepEqual((await commands(sha)).at(-1), ['sha256sum', './-example.tar.gz']);
  for (const name of ['', '../escape', 'dir/file', 'bad\nname']) {
    await assert.rejects(run(Sha.generate(client)({file, name})), /filename/);
  }
  await assert.rejects(run(Tar.create(client)({source: dir, name: 'ok.tar.gz', members: ['../escape']})), /relative/);
  await assert.rejects(run(Tar.create(client)({source: dir, name: 'ok.tar.gz', members: []})), /empty/);
});

test('upload graphs inject Secret values and never expand user paths', async () => {
  const fury = await run(Fury.uploadGraph(client)({packages: dir, files: ['example $HOME.deb'], account: 'example-team', runId: 'test-run', token}));
  const cmd = (await commands(fury)).at(-1);
  assert.equal(Fury.formPath('comma,semi;quote"'), 'comma,semi;quote\\"');
  assert.equal(await arg(fury.steps.find(s => s.name === 'withEnvVariable'), 'value'), 'test-run');
  assert.ok(cmd.includes('%FURY_TOKEN'));
  assert.ok(cmd.includes('{{FURY_TOKEN}}:'));
  assert.ok(cmd.includes('package=@"/packages/example $HOME.deb"'));
  assert.ok(cmd.includes('https://push.fury.io/example-team/'));
  assert.equal(fury.steps.filter(s => s.name === 'withSecretVariable').length, 1);
  assert.equal(fury.steps.at(-1).args.some(a => a.name === 'expand'), false);
  const github = await run(Github.uploadGraph(client)({assets: dir, files: ['example.tar.gz'], repository: 'owner/repo', tag: 'v1', token, runId: 'test-run', clobber: false}));
  assert.deepEqual((await commands(github)).at(-1), ['gh', 'release', 'upload', '--repo', 'owner/repo', '--', 'v1', '/assets/example.tar.gz']);
  for (const account of ['', '../other', 'host?query', 'team/name']) {
    await assert.rejects(run(Fury.uploadGraph(client)({packages: dir, files: ['pkg.deb'], account, runId: 'test-run', token})));
  }
  await assert.rejects(run(Github.uploadGraph(client)({assets: dir, files: [], repository: 'owner/repo', tag: 'v1', token, runId: 'test-run', clobber: false})), /required/);
});

test('Codecov uploads an explicit report with metadata, secret and cache key', async () => {
  const opts = {report: file, repository: 'owner/repo', commit: 'a'.repeat(40), branch: 'feature/$HOME', token, runId: '123-2-tests', pullRequest: '42', flags: ['webui', 'with spaces']};
  const graph = await run(Codecov.uploadGraph(client)(opts));
  assert.deepEqual((await commands(graph)).at(-1), ['codecovcli', 'upload-process', '--disable-search', '--disable-file-fixes', '--plugin', 'noop', '--fail-on-error', '--git-service', 'github', '--file', '/coverage/lcov.info', '--slug', opts.repository, '--sha', opts.commit, '--branch', opts.branch, '--name', opts.runId, '--pr', '42', '--flag', 'webui', '--flag', 'with spaces']);
  assert.equal(await arg(graph.steps.find(s => s.name === 'withSecretVariable'), 'name'), 'CODECOV_TOKEN');
  assert.equal(await arg(graph.steps.find(s => s.name === 'withEnvVariable'), 'value'), opts.runId);
  assert.equal(graph.steps.filter(s => s.name === 'withFile').length, 1);
  assert.equal(graph.steps.at(-1).args.some(a => a.name === 'expand'), false);
  const push = await run(Codecov.uploadGraph(client)({...opts, pullRequest: '', flags: []}));
  const cmd = (await commands(push)).at(-1);
  assert.ok(!cmd.includes('--pr') && !cmd.includes('--flag'));
  for (const key of ['repository', 'commit', 'branch', 'runId']) {
    await assert.rejects(run(Codecov.uploadGraph(client)({...opts, [key]: ''})), /required/);
  }
});
