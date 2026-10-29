# Music Player CI in PureScript

The portable GitHub Actions jobs are Dagger pipelines written in PureScript.
The CLI and engine are pinned to **Dagger 1.0.0-beta.21** in the root `mise.toml`.
The vendored SDK is a **personal, unofficial experimental SDK**, not an official
Dagger SDK. Its source and generated bindings are committed so a fresh clone
can run CI without a sibling checkout or a published SDK package.

## Reusable release modules

Five standalone modules are available under [`modules/`](modules/README.md):
`linux-packaging` (deb/rpm), `archive` (tar.gz/SHA-256), `gemfury`, and
`github-assets`, and `codecov` (coverage uploads). `CI.Release` reuses their shared PureScript implementations.
Linux workflows call the upload modules directly with Dagger Secrets, publishing
only the exported `artifacts/release` directory. Desktop metadata still comes
from the existing `dist/` templates; the standalone packaging script remains
available for developers.

## Run locally

Install [mise](https://mise.jdx.dev) and start Docker, then run from the repository
root:

```sh
mise trust
mise install
mise exec -- dagger api call -m .dagger/ci --help
mise exec -- dagger api call -m .dagger/ci web-check --source . export --path artifacts/webui
mise exec -- dagger api call -m .dagger/ci rust-coverage --source . export --path artifacts/lcov.info
mise exec -- dagger api call -m .dagger/ci clippy --source . export --path artifacts/rust-clippy-results.sarif
```

Checks default to `linux/amd64`, matching GitHub's Ubuntu runners. Add
`--platform linux/arm64` on an ARM development machine to run natively. Rust
coverage uses `/tmp/audio` fixtures and discards decoded audio via `fifo:/dev/null`.
Clippy findings remain advisory; its SARIF report is uploaded by GitHub Actions.

Build release artifacts without uploading anything:

```sh
mise exec -- dagger api call -m .dagger/ci linux-release \
  --source . --target x86_64-unknown-linux-gnu --version v0.4.5 \
  export --path artifacts/release
mise exec -- dagger api call -m .dagger/ci linux-release \
  --source . --target aarch64-unknown-linux-gnu --version v0.4.5 --desktop \
  export --path artifacts/desktop
```

The release directory contains the existing `.tar.gz` and `.tar.gz.sha256`
asset names. Desktop builds also return `dist/*.deb`, `dist/*.rpm`, and their
checksums. Each target selects its matching Linux container platform and DuckDB
archive. No cross-compilation sysroot is required. GitHub's ARM matrix still
uses native ARM runners.

Nix checks:

```sh
mise exec -- dagger api call -m .dagger/ci nix-flake --source .
mise exec -- dagger api call -m .dagger/ci nix-package --source . \
  --attribute music-player-desktop export --path artifacts/nix
```

The flake uses `flake.lock`; the nixpkgs packaging check intentionally follows
`nixos-unstable`, as before. `nix-package` exports a file-backed Nix store and a
`store-path` manifest. The workflow imports that closure into the runner's store,
allowing its existing Cachix action to publish it without passing credentials
into Dagger. Nix disables its inner sandbox because Dagger provides the outer
container boundary.

## Where the logic lives

| PureScript module | Responsibility |
| --- | --- |
| [CI.Base](ci/src/CI/Base.purs) | Platforms, clean source trees, container helpers |
| [CI.Web](ci/src/CI/Web.purs) | Frozen Bun install, typechecking, Vitest coverage, Vite, Storybook |
| [CI.Rust](ci/src/CI/Rust.purs) | Rust 1.98.0, DuckDB, system dependencies, cargo caches, coverage, Clippy |
| [CI.Release](ci/src/CI/Release.purs) | Linux daemon/desktop builds, archives, checksums, Debian/RPM packages |
| [CI.Nix](ci/src/CI/Nix.purs) | Linux flake and nixpkgs package checks |
| [Main](ci/src/Main.purs) | Typed Dagger function registration using named handlers and `ado` |

GitHub workflows retain their triggers, release selection, matrices, permissions,
and overwrite behavior. Native macOS/Windows compilation, macOS app signing/DMGs, and
FlakeHub publishing stay on their original runners.
SARIF and Cachix retain their existing upload steps. Rust and web UI coverage
uploads share the standalone Codecov module and remain non-blocking. Linux GitHub and Gemfury
credentials are passed to the upload modules as Secrets.

Release workflows check out pipeline tooling at `github.workflow_sha` and build
the requested release tree separately. The desktop workflow retains its original
behavior: the selected workflow ref supplies the code, while the input tag
selects the release to upload to.

## Edit the pipelines

Change the files under `ci/src/CI/` and register new public functions in
`ci/src/Main.purs`. Source and target arguments are typed; core Dagger objects
remain lazy until an artifact or scalar is requested.

If you change dependencies or the SDK, regenerate from the module directory:

```sh
cd .dagger/ci
mise exec -- dagger generate -y
```

Commit `ci/dagger-sdk/`, module TOML files, and the root `dagger.lock` alongside
your source. Beta module execution uses the generated bindings already on disk.
For local compilation, install Node.js 22+, PureScript 0.15.16, and Spago 1.0.5,
then run `spago build` in `ci/`. Those tools are installed inside the module's
Dagger runtime during CI; they are not required on the GitHub host.

The reusable setup action installs only the pinned Dagger CLI. Cache mounts live
in the engine and are reused when that engine persists; fresh hosted runners
start cold unless a persistent engine is configured. Build inputs exclude host
caches, `.env` files, and CI implementation files. Never edit generated bindings
by hand.

## Standalone jobs

The old runner and all of its Deno/FluentCI dependencies have been removed.
The replacement commands are entirely local Dagger/PureScript code:

| Previous job | Dagger function | Result |
| --- | --- | --- |
| `test` | `test` | Integration coverage after migration, fixture scanning, and daemon startup |
| `build` | `build` | Release archive and checksum |
| `clippy` | `clippy` | SARIF report |
| Default test then build | `pipeline` | Coverage plus release artifacts |

```sh
./.dagger/run                 # test, then build
./.dagger/run test clippy     # selected jobs, in order
TAG=v0.4.5 ./.dagger/run build
PLATFORM=linux/arm64 TARGET=aarch64-unknown-linux-gnu ./.dagger/run
```

`SOURCE`, `PLATFORM`, `TARGET`, and `TAG` are explicit runner inputs. Without
`TAG`, standalone builds use `latest`; release workflows pass their release
version. Artifacts are exported to `artifacts/`. The default integration pipeline
is also available as a manual GitHub Actions workflow. It waits for daemon
readiness and cleans up the process even if tests fail.

`CI.Integration` implements database migration and live daemon testing;
`CI.Pipeline` composes tests and builds. No remote pipeline imports, Deno runtime,
FluentCI SDK, or legacy JSON configuration remain.

## Validate without an engine

```sh
cd .dagger/ci
spago build -p music-player-ci --pedantic-packages
node --test checks/*.test.mjs
cd ../..
python3 .dagger/check-modules.py
python3 .dagger/check.py
python3 .dagger/check-uploads.py
```

The Dagger module checks workflow runs these offline checks and actionlint.
Upload shell blocks are tested with a fake executable; no engine or external
publication is used. These checks do not replace end-to-end container validation.
