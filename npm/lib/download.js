// Fetch the music-player binary for this platform from a GitHub release,
// verify its SHA-256 when the release publishes one, and unpack it next to
// the bin shim. Used twice: by the postinstall script, and lazily by the
// shim as a fallback when installs ran with --ignore-scripts.
//
// Which release: the latest one by default, or the tag named by the
// MUSIC_PLAYER_VERSION environment variable (with or without the leading
// "v"), e.g. MUSIC_PLAYER_VERSION=v0.2.0 npx music-player

"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const REPO = "tsirysndr/music-player";
const VERSION_ENV = "MUSIC_PLAYER_VERSION";

const exeSuffix = os.platform() === "win32" ? ".exe" : "";

/** Where the downloaded binary lives (inside the installed package). */
const binaryPath = path.join(__dirname, "..", "bin", `music-player-bin${exeSuffix}`);

/** Records which release tag the cached binary came from. */
const stampPath = path.join(__dirname, "..", "bin", ".release-tag");

/** The release target triple for this machine, or an explanatory error. */
function resolveTarget() {
  const platform = os.platform();
  const arch = os.arch();
  if (platform === "darwin" && arch === "arm64") return "aarch64-apple-darwin";
  if (platform === "darwin" && arch === "x64") return "x86_64-apple-darwin";
  if (platform === "linux" && arch === "x64") return "x86_64-unknown-linux-gnu";
  if (platform === "linux" && arch === "arm64") return "aarch64-unknown-linux-gnu";
  if (platform === "linux" && arch === "arm") return "armv7-unknown-linux-gnueabihf";
  if (platform === "win32" && arch === "x64") return "x86_64-pc-windows-gnu";
  throw new Error(
    `unsupported platform ${platform}/${arch} — build from source: ` +
      `cargo install --git https://github.com/${REPO}`,
  );
}

async function fetchBytes(url) {
  const res = await fetch(url, {
    redirect: "follow",
    headers: { "User-Agent": "music-player-npm" },
  });
  if (!res.ok) {
    const err = new Error(`GET ${url} -> HTTP ${res.status}`);
    err.status = res.status;
    throw err;
  }
  return Buffer.from(await res.arrayBuffer());
}

/** The release tag to install: $MUSIC_PLAYER_VERSION verbatim, or the newest
 *  release that actually publishes this platform's CLI asset (releases are
 *  occasionally partial, so blindly taking "latest" can 404). */
async function resolveTag(target) {
  const override = process.env[VERSION_ENV];
  if (override && override.trim() !== "") {
    const v = override.trim();
    return v.startsWith("v") ? v : `v${v}`;
  }
  const body = await fetchBytes(`https://api.github.com/repos/${REPO}/releases?per_page=15`);
  const releases = JSON.parse(body.toString("utf8"));
  for (const release of releases) {
    const wanted = assetName(release.tag_name, target);
    if ((release.assets || []).some((a) => a.name === wanted)) {
      return release.tag_name;
    }
  }
  throw new Error(`no recent release publishes a ${target} build`);
}

/** Asset name for a tag+target. The Windows asset is published without a tag. */
function assetName(tag, target) {
  if (target === "x86_64-pc-windows-gnu") {
    return `music-player_${target}.tar.gz`;
  }
  return `music-player_${tag}_${target}.tar.gz`;
}

/** Extract the tarball and locate the binary wherever the layout put it —
 *  macOS releases store it at the archive root, Linux ones under
 *  target/<triple>/release/. */
function extractBinary(tarPath, tmp) {
  const out = spawnSync("tar", ["-xzf", tarPath, "-C", tmp], {
    stdio: ["ignore", "ignore", "inherit"],
  });
  if (out.status !== 0) throw new Error(`tar extraction failed for ${tarPath}`);
  const wanted = `music-player${exeSuffix}`;
  const stack = [tmp];
  while (stack.length > 0) {
    const dir = stack.pop();
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const p = path.join(dir, entry.name);
      if (entry.isDirectory()) stack.push(p);
      else if (entry.name === wanted) return p;
    }
  }
  throw new Error(`no ${wanted} found inside the release archive`);
}

/** Download + verify + unpack. Idempotent: returns fast when the cached
 *  binary matches the requested release (any cached binary, when no explicit
 *  version was requested and one is already present). */
async function ensureBinary() {
  const override = process.env[VERSION_ENV];
  if (fs.existsSync(binaryPath)) {
    if (!override || override.trim() === "") return binaryPath;
    const cached = fs.existsSync(stampPath)
      ? fs.readFileSync(stampPath, "utf8").trim()
      : "";
    const wanted = override.trim().startsWith("v") ? override.trim() : `v${override.trim()}`;
    if (cached === wanted) return binaryPath;
  }

  const target = resolveTarget();
  const tag = await resolveTag(target);
  const asset = assetName(tag, target);
  const base = `https://github.com/${REPO}/releases/download/${tag}`;

  console.error(`downloading music-player ${tag} (${target}) from github.com/${REPO}…`);
  const tarball = await fetchBytes(`${base}/${asset}`);

  // Releases publish `<sha256>  <asset>` next to each tarball; verify when
  // present, warn (older releases may lack it) when not.
  try {
    const sumLine = (await fetchBytes(`${base}/${asset}.sha256`)).toString("utf8");
    const expected = sumLine.trim().split(/\s+/)[0];
    const actual = crypto.createHash("sha256").update(tarball).digest("hex");
    if (!expected || expected.toLowerCase() !== actual) {
      throw new Error(`checksum mismatch for ${asset}: expected ${expected}, got ${actual}`);
    }
  } catch (err) {
    if (err.status !== 404) throw err;
    console.error(`no checksum published for ${asset}; skipping verification`);
  }

  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "music-player-npm-"));
  try {
    const tarPath = path.join(tmp, asset);
    fs.writeFileSync(tarPath, tarball);
    const extracted = extractBinary(tarPath, tmp);
    fs.mkdirSync(path.dirname(binaryPath), { recursive: true });
    fs.copyFileSync(extracted, binaryPath);
    fs.chmodSync(binaryPath, 0o755);
    fs.writeFileSync(stampPath, `${tag}\n`);
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
  return binaryPath;
}

module.exports = { binaryPath, ensureBinary };
