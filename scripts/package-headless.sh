#!/usr/bin/env bash
set -euo pipefail
: "${RELEASE_VERSION:?}" "${TARGET:?}" "${ARCH:?}" "${SOURCE_REVISION:?}"
binary="$(pwd)/target/$TARGET/release/music-player"
bash dist/package-linux.sh all "$RELEASE_VERSION" "$ARCH" "$(dirname "$binary")" cli
out="$(pwd)/dist/build/release"
evidence="$(pwd)/dist/build/evidence"
mkdir -p "$out" "$evidence" "dist/build/image/binaries/$ARCH"
cp dist/music-player-cli_*.deb dist/music-player-cli-*.rpm "$out/"
install -m0755 "$binary" "dist/build/image/binaries/$ARCH/music-player"
cp LICENSE dist/build/image/
asset="music-player_v${RELEASE_VERSION}_$TARGET"
tar -czf "$out/$asset.tar.gz" -C "$(dirname "$binary")" music-player -C "$(pwd)" LICENSE
ldd "$binary" | tee "$evidence/ldd-$ARCH.txt"
if grep -q 'not found' "$evidence/ldd-$ARCH.txt"; then exit 1; fi
readelf -d "$binary" > "$evidence/elf-$ARCH.txt"
readelf --version-info "$binary" >> "$evidence/elf-$ARCH.txt"
deb=(dist/music-player-cli_*.deb)
rpm=(dist/music-player-cli-*.rpm)
[[ ${#deb[@]} == 1 && ${#rpm[@]} == 1 ]]
[[ $(dpkg-deb -f "${deb[0]}" Package) == music-player-cli ]]
[[ $(dpkg-deb -f "${deb[0]}" Conflicts) == music-player ]]
[[ $(dpkg-deb -f "${deb[0]}" Architecture) == "$ARCH" ]]
package_version=${RELEASE_VERSION//-/\~}
[[ $(dpkg-deb -f "${deb[0]}" Version) == "$package_version" ]]
[[ $(rpm -qp --qf '%{VERSION}' "${rpm[0]}") == "$package_version" ]]
if [[ "$RELEASE_VERSION" == *-* ]]; then
  dpkg --compare-versions "$package_version" lt "${RELEASE_VERSION%%-*}"
fi
dpkg-deb -f "${deb[0]}" > "$evidence/deb-$ARCH.txt"
rpm -qp --requires "${rpm[0]}" > "$evidence/rpm-$ARCH.txt"
rpm -qp --conflicts "${rpm[0]}" | grep -qx music-player
mkdir -p dist/build/verify-deb dist/build/verify-rpm dist/build/verify-tar
dpkg-deb -x "${deb[0]}" dist/build/verify-deb
rpm2cpio "${rpm[0]}" | (cd dist/build/verify-rpm && cpio -idm --quiet)
tar -xzf "$out/$asset.tar.gz" -C dist/build/verify-tar
cmp "$binary" dist/build/verify-deb/usr/bin/music-player
cmp "$binary" dist/build/verify-rpm/usr/bin/music-player
cmp "$binary" dist/build/verify-tar/music-player
# Fail if desktop files or a second executable leak into a CLI package.
for root in dist/build/verify-deb dist/build/verify-rpm; do
  [[ $(find "$root/usr/bin" -type f | wc -l) == 1 ]]
  if find "$root" -type f | grep -E '/applications/|/icons/|music-player-desktop'; then exit 1; fi
done
python3 - "$binary" "$out/build-$ARCH.json" <<'PY'
import hashlib, json, os, subprocess, sys
from pathlib import Path
binary, output = map(Path, sys.argv[1:])
metadata = {key: os.environ[key] for key in ('SOURCE_REVISION', 'RELEASE_VERSION', 'TARGET', 'ARCH')}
metadata.update(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                rust=subprocess.check_output(['rustc', '+1.98.0', '--version'], text=True).strip(),
                bun=subprocess.check_output(['bun', '--version'], text=True).strip(),
                build_baseline='Ubuntu 24.04', deb_baseline='Ubuntu 24.04',
                runtime_baseline='Debian 13 (trixie-slim)',
                version=subprocess.check_output([str(binary), '--version'], text=True).strip())
output.write_text(json.dumps(metadata, indent=2) + '\n')
PY
cp "$evidence/"*.txt "$out/"
(cd "$out" && for file in *; do sha256sum "$file" > "$file.sha256"; done)
