import assert from 'node:assert/strict';
import { once } from 'node:events';
import { createServer } from 'node:http';
import { setImmediate } from 'node:timers/promises';
import test from 'node:test';

// Exercise both the SDK source and the copy consumed by the CI module.
for (const path of ['../../sdk/purescript', '../dagger-sdk']) {
  const runtime = await import(`${path}/src/Dagger/Internal/Runtime.js`);
  const start = (session, query = '{ value }') => {
    let cancel;
    const result = new Promise((resolve, reject) => {
      cancel = runtime.rawImpl(session)(query)({})(error => () => reject(error))(value => () => resolve(value))();
    });
    return { result, cancel };
  };
  async function fixture(t, handler) {
    const server = createServer(handler);
    server.listen(0, '127.0.0.1');
    await once(server, 'listening');
    t.after(() => { server.closeAllConnections(); server.close(); });
    return { port: server.address().port, token: 'test', timeoutMs: 0 };
  }

  test(`${path}: default connection survives five minutes without response headers`, async t => {
    let received;
    const ready = new Promise(resolve => { received = resolve; });
    const config = await fixture(t, (req, res) => { req.resume(); received({ req, res }); });
    for (const [name, value] of Object.entries({ DAGGER_SESSION_PORT: String(config.port), DAGGER_SESSION_TOKEN: config.token })) {
      const previous = process.env[name];
      process.env[name] = value;
      t.after(() => { if (previous === undefined) delete process.env[name]; else process.env[name] = previous; });
    }
    t.mock.timers.enable({ apis: ['setTimeout'] });
    const pending = start(runtime.connect());
    const { req, res } = await ready;
    assert.equal(req.url, '/query');
    assert.equal(req.headers.authorization, `Basic ${Buffer.from('test:').toString('base64')}`);
    t.mock.timers.tick(300001);
    await setImmediate();
    res.end(JSON.stringify({ data: { value: 'built' } }));
    assert.deepEqual(await pending.result, { value: 'built' });
  });

  test(`${path}: explicit deadline aborts while waiting for the body`, async t => {
    let received;
    const ready = new Promise(resolve => { received = resolve; });
    const config = await fixture(t, (req, res) => {
      req.resume(); res.writeHead(200); res.write('{'); received();
    });
    t.mock.timers.enable({ apis: ['setTimeout'] });
    const pending = start(runtime.connectWith({ ...config, timeoutMs: 100 })());
    const rejected = assert.rejects(pending.result, /Dagger request timed out after 100ms/);
    await ready;
    t.mock.timers.tick(100);
    await rejected;
  });

  test(`${path}: cancellation closes the request without callbacks`, async t => {
    let received;
    const ready = new Promise(resolve => { received = resolve; });
    const config = await fixture(t, (req, res) => { req.resume(); received(res); });
    const pending = start(runtime.connectWith(config)());
    let completed = false;
    pending.result.then(() => { completed = true; }, () => { completed = true; });
    const res = await ready;
    const closed = once(res, 'close');
    pending.cancel();
    await closed;
    await setImmediate();
    assert.equal(completed, false);
  });

  test(`${path}: HTTP and GraphQL errors retain diagnostics`, async t => {
    let status = 503;
    let body = {};
    const config = await fixture(t, (req, res) => { req.resume(); res.writeHead(status); res.end(JSON.stringify(body)); });
    const session = runtime.connectWith(config)();
    await assert.rejects(start(session).result, /Dagger HTTP error 503/);
    status = 200;
    body = { errors: [{ message: 'compiler failed', extensions: { _type: 'EXEC_ERROR', exitCode: 1 } }] };
    await assert.rejects(start(session).result, error => error.name === 'DaggerExecError' && error.extensions.exitCode === 1);
    for (const timeoutMs of [-1, 0.5, NaN, 2147483648]) {
      assert.throws(runtime.connectWith({ ...config, timeoutMs }), /timeout/);
    }
  });
}
