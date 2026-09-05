#!/usr/bin/env node
// Thin launcher for the native music-player binary. Normally postinstall has
// already fetched it; when install scripts were skipped (--ignore-scripts,
// some CI setups) it is fetched here on first run instead, so the command
// works either way.

"use strict";

const { spawn } = require("node:child_process");
const { ensureBinary } = require("../lib/download");

async function main() {
  const bin = await ensureBinary();
  const child = spawn(bin, process.argv.slice(2), { stdio: "inherit" });
  // Forward termination signals so `kill <npm-shim-pid>` stops the daemon too
  // (Ctrl-C already reaches the child via the shared process group).
  for (const sig of ["SIGINT", "SIGTERM", "SIGHUP"]) {
    process.on(sig, () => child.kill(sig));
  }
  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    } else {
      process.exitCode = code ?? 1;
    }
  });
}

main().catch((err) => {
  console.error(`music-player: ${err.message}`);
  process.exitCode = 1;
});
