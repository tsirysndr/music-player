#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 <deb|rpm|all> <version> <architecture> <binary-directory>" >&2
  echo "  Debian architectures: amd64, arm64" >&2
  echo "  RPM architectures:    x86_64, aarch64" >&2
  exit 2
}

[[ $# -eq 4 ]] || usage

format=$1
version=${2#v}
architecture=$3
binary_dir=$4
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)

[[ "$format" == deb || "$format" == rpm || "$format" == all ]] || usage
[[ "$version" =~ ^[0-9][0-9A-Za-z.+~-]*$ ]] || {
  echo "invalid package version: $version" >&2
  exit 2
}

for binary in music-player music-player-desktop; do
  [[ -x "$binary_dir/$binary" ]] || {
    echo "missing executable: $binary_dir/$binary" >&2
    exit 1
  }
done

stage_root="$script_dir/build/root"
rm -rf "$script_dir/build"
install -d \
  "$stage_root/usr/bin" \
  "$stage_root/usr/share/applications" \
  "$stage_root/usr/share/icons/hicolor/scalable/apps" \
  "$stage_root/usr/share/licenses/music-player"
install -m 0755 "$binary_dir/music-player" "$stage_root/usr/bin/music-player"
install -m 0755 "$binary_dir/music-player-desktop" "$stage_root/usr/bin/music-player-desktop"
install -m 0644 "$script_dir/music-player.desktop" "$stage_root/usr/share/applications/music-player.desktop"
install -m 0644 "$repo_root/desktop/assets/icon.svg" "$stage_root/usr/share/icons/hicolor/scalable/apps/music-player.svg"
install -m 0644 "$repo_root/LICENSE" "$stage_root/usr/share/licenses/music-player/LICENSE"

build_deb() {
  local deb_arch=$architecture
  case "$deb_arch" in
    amd64|arm64) ;;
    x86_64) deb_arch=amd64 ;;
    aarch64) deb_arch=arm64 ;;
    *) echo "unsupported Debian architecture: $architecture" >&2; exit 2 ;;
  esac

  local package_root="$script_dir/build/debian"
  cp -a "$stage_root/." "$package_root/"
  install -d "$package_root/DEBIAN"
  sed \
    -e "s/@VERSION@/$version/g" \
    -e "s/@DEB_ARCH@/$deb_arch/g" \
    "$script_dir/debian-control.in" > "$package_root/DEBIAN/control"
  dpkg-deb --build --root-owner-group "$package_root" \
    "$script_dir/music-player_${version}_${deb_arch}.deb"
}

build_rpm() {
  local rpm_arch=$architecture
  local rpm_version=${version//-/~}
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
    "$script_dir/music-player.spec.in" > "$rpm_top/SPECS/music-player.spec"
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
  sha256sum "$package" > "$package.sha256"
done
