import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { runAff } from '../output/Effect.Aff/index.js';
import { Left } from '../output/Data.Either/index.js';
import { connectWith } from '../output/Dagger/index.js';
import * as Q from '../output/Dagger.Query/index.js';
import * as Release from '../output/CI.Release/index.js';
const client = connectWith({port: 1, token: 'offline', timeoutMs: 1})();
const run = aff => new Promise((resolve,reject) => runAff(r => () => r instanceof Left ? reject(r.value0) : resolve(r.value0))(aff)());
const arg = (step,name) => run(step.args.find(a => a.name === name).value);

test('release version and platform validation precedes builds', async () => {
  for (const version of ['latest','v1.2.3','1.2.3-rc1','2026.10.28']) assert.equal(Release.validVersion(version), true);
  for (const version of ['', 'v', 'vlatest', '../escape', '1\nBad: field', '$(touch file)', 'v1/evil']) {
    assert.equal(Release.validVersion(version), false);
    await assert.rejects(run(Release.linux(client)({source: Q.directory(client), target: 'x86_64-unknown-linux-gnu', version, desktop: false})), /version/);
  }
  await assert.rejects(run(Release.linux(client)({source: Q.directory(client), target: 'unsupported', version: '1.0', desktop: false})), /support/);
  await assert.rejects(run(Release.linux(client)({source: Q.directory(client), target: 'x86_64-unknown-linux-gnu', version: 'latest', desktop: true})), /numeric/);
});

test('desktop templates preserve metadata and architecture conventions', () => {
  const control = readFileSync(new URL('../../../dist/debian-control.in',import.meta.url),'utf8');
  const spec = readFileSync(new URL('../../../dist/music-player.spec.in',import.meta.url),'utf8');
  assert.equal(Release.metadataVersion('v1.2.3-rc1'), '1.2.3-rc1');
  for (const [debArch,rpmArch] of [['amd64','x86_64'],['arm64','aarch64']]) {
    const deb=Release.stampControl('1.2.3-rc1')(debArch)(control);
    const rpm=Release.stampSpec('1.2.3-rc1')(rpmArch)(spec);
    assert.match(deb,/Version: 1\.2\.3-rc1/);
    assert.ok(deb.includes(`Architecture: ${debArch}`));
    assert.match(rpm,/Version:\s+1\.2\.3~rc1/);
    assert.ok(rpm.includes(rpmArch));
    assert.ok(rpm.includes('%{_sourcedir}/root/.'));
    assert.doesNotMatch(deb+rpm, /@VERSION@|@DEB_ARCH@|@RPM_ARCH@/);
  }
});

test('desktop staging installs both binaries and integration assets with fixed permissions', async () => {
  const graph=Release.stageDesktop('target/aarch64-unknown-linux-gnu/release')(Q.container(client));
  const commands=await Promise.all(graph.steps.filter(s=>s.name==='withExec').map(s=>arg(s,'args')));
  assert.deepEqual(commands[1],['install','-m','0755','target/aarch64-unknown-linux-gnu/release/music-player','target/aarch64-unknown-linux-gnu/release/music-player-desktop','/stage/usr/bin/']);
  assert.equal(commands.filter(c=>c.includes('0644')).length,3);
  assert.ok(commands.every(c=>c[0]==='install'));
});
