#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 <deb|rpm|all> <version> <architecture> <binary-directory> [full|cli]" >&2
  echo "  Debian architectures: amd64, arm64" >&2
  echo "  RPM architectures:    x86_64, aarch64" >&2
  exit 2
}

[[ $# -eq 4 || $# -eq 5 ]] || usage

format=$1
version=${2#v}
architecture=$3
binary_dir=$4
variant=${5:-full}
[[ "$variant" == full || "$variant" == cli ]] || usage
package_name=music-player
binaries=(music-player music-player-desktop)
control_template=debian-control.in
rpm_template=music-player.spec.in
if [[ "$variant" == cli ]]; then
  package_name=music-player-cli
  binaries=(music-player)
  control_template=debian-control-cli.in
  rpm_template=music-player-cli.spec.in
fi
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)

[[ "$format" == deb || "$format" == rpm || "$format" == all ]] || usage
[[ "$version" =~ ^[0-9][0-9A-Za-z.+~-]*$ ]] || {
  echo "invalid package version: $version" >&2
  exit 2
}

for binary in "${binaries[@]}"; do
  [[ -x "$binary_dir/$binary" ]] || {
    echo "missing executable: $binary_dir/$binary" >&2
    exit 1
  }
done

stage_root="$script_dir/build/root"
rm -rf "$script_dir/build"
install -d \
  "$stage_root/usr/bin" \
  "$stage_root/usr/share/licenses/$package_name"
install -m 0755 "$binary_dir/music-player" "$stage_root/usr/bin/music-player"
if [[ "$variant" == full ]]; then
  install -d "$stage_root/usr/share/applications" "$stage_root/usr/share/icons/hicolor/scalable/apps"
  install -m 0755 "$binary_dir/music-player-desktop" "$stage_root/usr/bin/music-player-desktop"
  install -m 0644 "$script_dir/music-player.desktop" "$stage_root/usr/share/applications/music-player.desktop"
  install -m 0644 "$repo_root/desktop/assets/icon.svg" "$stage_root/usr/share/icons/hicolor/scalable/apps/music-player.svg"
fi
install -m 0644 "$repo_root/LICENSE" "$stage_root/usr/share/licenses/$package_name/LICENSE"

build_deb() {
  local deb_arch=$architecture
  # Debian prereleases must sort before the final version, not after it.
  local deb_version=${version//-/\~}
  case "$deb_arch" in
    amd64|arm64) ;;
    x86_64) deb_arch=amd64 ;;
    aarch64) deb_arch=arm64 ;;
    *) echo "unsupported Debian architecture: $architecture" >&2; exit 2 ;;
  esac

  local package_root="$script_dir/build/debian"
  cp -a "$stage_root/." "$package_root/"
  install -d "$package_root/DEBIAN"
  local depends=""
  if [[ "$variant" == cli ]]; then
    install -Dm0644 "$script_dir/systemd/music-player.service" "$package_root/usr/lib/systemd/system/music-player.service"
    install -Dm0644 "$script_dir/debian/music-player.default" "$package_root/etc/default/music-player"
    printf '/etc/default/music-player\n' > "$package_root/DEBIAN/conffiles"
    for maintscript in postinst prerm postrm; do
      install -m0755 "$script_dir/debian/$maintscript" "$package_root/DEBIAN/$maintscript"
    done
    # dpkg-shlibdeps requires a Debian source-control context, even for a
    # prebuilt executable. Resolve dependencies on the native build runner.
    mkdir -p "$script_dir/build/shlibdeps/debian"
    printf 'Source: music-player-cli\nSection: sound\nPriority: optional\nMaintainer: Music Player contributors <tsiry.sndr@rocksky.app>\nStandards-Version: 4.6.2\n\nPackage: music-player-cli\nArchitecture: any\nDescription: music player\n' > "$script_dir/build/shlibdeps/debian/control"
    depends=$(cd "$script_dir/build/shlibdeps" && dpkg-shlibdeps -O -e"$package_root/usr/bin/music-player")
    depends=${depends#shlibs:Depends=}
    [[ -n "$depends" ]] || { echo "empty shared-library dependencies" >&2; exit 1; }
  fi
  sed \
    -e "s/@DEPENDS@/$depends/g" \
    -e "s/@VERSION@/$deb_version/g" \
    -e "s/@DEB_ARCH@/$deb_arch/g" \
    "$script_dir/$control_template" > "$package_root/DEBIAN/control"
  dpkg-deb --build --root-owner-group "$package_root" \
    "$script_dir/${package_name}_${version}_${deb_arch}.deb"
}

build_rpm() {
  local rpm_arch=$architecture
  # Keep the RPM prerelease separator literal; unquoted ~ expands to HOME.
  local rpm_version=${version//-/\~}
  case "$rpm_arch" in
    x86_64|aarch64) ;;
    amd64) rpm_arch=x86_64 ;;
    arm64) rpm_arch=aarch64 ;;
    *) echo "unsupported RPM architecture: $architecture" >&2; exit 2 ;;
  esac

  local rpm_top="$script_dir/rpmbuild"
  rm -rf "$rpm_top"
  install -d "$rpm_top"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}
  cp -a "$stage_root" "$rpm_top/SOURCES/root"
  sed \
    -e "s/@VERSION@/$rpm_version/g" \
    -e "s/@RPM_ARCH@/$rpm_arch/g" \
    "$script_dir/$rpm_template" > "$rpm_top/SPECS/music-player.spec"
  rpmbuild --define "_topdir $rpm_top" --target "$rpm_arch" \
    -bb "$rpm_top/SPECS/music-player.spec"
  find "$rpm_top/RPMS" -type f -name '*.rpm' -exec mv -f {} "$script_dir/" \;
}

case "$format" in
  deb) build_deb ;;
  rpm) build_rpm ;;
  all) build_deb; build_rpm ;;
esac

for package in "$script_dir"/*.deb "$script_dir"/*.rpm; do
  [[ -e "$package" ]] || continue
  (cd "$script_dir" && sha256sum "$(basename "$package")") > "$package.sha256"
done
