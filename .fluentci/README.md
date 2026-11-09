# FluentCI pipelines

Requires FluentCI 0.17.3+, Node 24, Bun 1.4.3, PureScript 0.15.16, and Spago 1.0.5.
The toolchain composite generates `.sdk/` from the installed CLI in a temporary
workspace, then the CLI compiles the local PureScript component on first use.
For local development, generate a temporary `fluentci init` PureScript project
and copy its module `.sdk/` beside `.fluentci/ci/package.json` before first use. Commit `spago.lock` and `package-lock.json`; generated SDKs and Wasm stay ignored.

```sh
fluentci run ci --help
fluentci run ci web-check
fluentci run ci dependencies
fluentci run ci coverage
fluentci run ci release --source . --target aarch64-apple-darwin --version v0.4.7 --desktop=false
fluentci run ci nix-build
fluentci run ci check
```

Install Rust 1.98.0 with `llvm-tools-preview` for coverage and `clippy` for SARIF.
`dependencies` installs native libraries using sudo apt on Linux or Homebrew on
macOS. Builds now run on the GitHub host; Linux releases therefore inherit the
runner's glibc baseline rather than the former Debian Bookworm container baseline.
ARM Linux releases require the matching native runner.

Workflows use the published `setup-node`, `setup-bun`, `setup-purescript`,
`setup-rust`, `setup-codecov`, `setup-gemfury`, `setup-determinate-nix`, and
`setup-flakehub-push` modules. Setup stdout is piped through `github-env.mjs` to
persist the returned environment between separate GitHub steps. GitHub's Bash
pipefail setting propagates installation failures.

The local component retains LCOV and SARIF paths under `artifacts/`, web coverage
and builds under `artifacts/webui/`, and release archives, DMGs, Debian/RPM
packages and checksums under `artifacts/release/`. Release tooling stays at the
workflow revision; CLI releases can build an older checkout in `release-source/`.
Desktop releases build the workflow checkout, preserving their prior behavior.

Codecov uploads are advisory and read `CODECOV_TOKEN`. Gemfury reads `FURY_TOKEN`,
and `FURY_ACCOUNT`; a missing
token skips uploads. Curl receives credential environment references rather than
secret values in its arguments. Cachix pushes require `CACHIX_AUTH_TOKEN` and
otherwise skip. FlakeHub uses the workflow's GitHub OIDC permission.

Release staging, DuckDB downloads, app signing, Debian/RPM assembly, checksums,
and uploads use PureScript control flow with literal executable arguments. No
new Bash scripts or embedded Bash programs implement these jobs.

The old `.dagger/` pipelines remain available for manual comparison. GitHub no
longer runs them. The native Windows release is unchanged because the FluentCI
setup action rejects Windows.
