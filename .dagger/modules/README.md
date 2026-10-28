# Reusable release modules

These four Dagger modules accept built files, staged directories and explicit
metadata. They do not require Music Player source, run Cargo, download a Music Player
release, or assume Music Player package names.

| Module | Functions | Inputs and outputs |
| --- | --- | --- |
| `linux-packaging` | `deb`, `rpm` | Debian root → `.deb` File; RPM sources + spec → Directory of RPMs |
| `archive` | `tar-gz`, `sha256` | Directory → `.tar.gz` File; File → SHA-256 sidecar File |
| `gemfury` | `upload` | Package directory, explicit filenames, account and Secret → upload report |
| `github-assets` | `upload` | Asset directory, explicit filenames, repository, tag and Secret → upload report |

The PureScript implementations live in `../lib/src/Release`. Standalone modules
and Music Player CI import this same library. Each standalone module has its own
Dagger manifest, typed entrypoint and locked Spago workspace. The manifests
include the shared library and the existing vendored SDK in `../ci/dagger-sdk`;
keep these sibling paths when copying the modules to another repository. Git
module references must point into this repository so those shared inputs remain
available. No SDK copies or SDK runtime changes are needed.

## Packaging and archives

The following are invocation examples; run them only on your intended Dagger
engine. This repository pins Dagger `1.0.0-beta.21`.

```sh
# package-root contains DEBIAN/control and the installed filesystem tree.
# control owns the name, version, architecture, dependencies and maintainer.
mise exec -- dagger api call -m .dagger/modules/linux-packaging deb \
  --root package-root --name example_1.0.0_amd64.deb \
  export --path artifacts/example_1.0.0_amd64.deb

# sources maps directly to %{_sourcedir}; the spec owns all RPM metadata.
# RPM output retains rpmbuild's architecture subdirectories.
mise exec -- dagger api call -m .dagger/modules/linux-packaging rpm \
  --sources rpm-sources --spec example.spec --target x86_64 \
  export --path artifacts/rpms

# members defaults to the whole source directory ('.').
mise exec -- dagger api call -m .dagger/modules/archive tar-gz \
  --source staged-files --name example.tar.gz \
  export --path artifacts/example.tar.gz

# name is the published filename, independent of the input file's local name.
mise exec -- dagger api call -m .dagger/modules/archive sha256 \
  --file artifacts/example.tar.gz --name example.tar.gz \
  export --path artifacts/example.tar.gz.sha256
```

Packaging uses Ubuntu 24.04 with `dpkg-dev` or `rpm` and `build-essential`.
The RPM spec should package prebuilt inputs or use those available tools;
project-specific build dependencies belong in the upstream build pipeline.
Debian packages use root ownership without requiring root-owned host files.
Archive creation preserves the staged files and does not normalize timestamps.
Checksum generation is separate so it also works for APKs, libraries and other
assets. Sidecars can be verified with `sha256sum -c` from the artifact directory.

## Uploads

```sh
mise exec -- dagger api call -m .dagger/modules/gemfury upload \
  --packages artifacts --files example_1.0.0_amd64.deb --account example-team \
  --token env:FURY_TOKEN --run-id "$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"

mise exec -- dagger api call -m .dagger/modules/github-assets upload \
  --assets artifacts --files example.tar.gz --repository owner/repository \
  --tag v1.0.0 --token env:GH_TOKEN \
  --run-id "$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
```

`files` is an array of paths relative to the supplied directory, not a shell
pattern. Use the CLI's array argument syntax for multiple files. The GitHub
release must already exist; `clobber` defaults to false. GitHub paths containing
`#` are rejected because `gh` reserves that character for asset labels.

Both upload functions take Dagger `Secret` arguments and force their final
container operation before returning success. A required `run-id` changes the
upload graph's cache key: supply a unique value for each intended upload attempt
so a previous successful upload cannot mask a later remote change. Credentials
are injected only into upload containers. Curl imports the token from the secret
environment variable and expands only its authentication option, not filenames.
No uploads are performed by the offline tests.

## Existing CI and verification

The existing `.dagger/ci` public functions and release filenames remain unchanged.
`CI.Release` stages the Music Player binaries, desktop entry, icon and license,
and stamps the existing Debian/RPM templates. It calls the shared packaging,
archive and checksum implementations instead of `dist/package-linux.sh`.
The script remains available for standalone developer use. Package checksum
sidecars now contain portable basenames rather than absolute build paths.

Linux daemon, ARM and desktop workflows export to `artifacts/release` and call
`github-assets upload` directly. Desktop package publishing calls `gemfury upload`
when its secrets are available. Upload cache keys include the Actions run,
attempt, job and target. macOS and Windows retain their native publishing steps.

These reusable implementations are vendored from the matching Rockbox module
split, with no runtime dependency on a sibling checkout. Keep shared release
changes synchronized between the two repositories when updating them.

```sh
# Compile all standalone entrypoints using only declared shared inputs.
python3 .dagger/check-modules.py
cd .dagger/ci
spago build -p music-player-ci --pedantic-packages
node --test checks/*.test.mjs
cd ../..
python3 .dagger/check.py
python3 .dagger/check-uploads.py
```

`check-modules.py` stages temporary workspaces, compiles without Dagger, and
refreshes module lockfiles if their dependencies changed. The workflow runs these
same checks. Graph tests cover generic packaging arguments, literal archive
members and filenames, path validation, credential injection and upload cache
keys. These checks do not establish end-to-end package or upload success.
