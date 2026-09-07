#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 <app|dmg|all> <version> <binary-directory>" >&2
  echo "  Builds dist/build/macos/Music Player.app and dist/build/Music Player.dmg." >&2
  echo "  Callers own the release asset names; this script owns the bundle." >&2
  exit 2
}

[[ $# -eq 3 ]] || usage

format=$1
version=${2#v}
binary_dir=$3
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)

[[ "$format" == app || "$format" == dmg || "$format" == all ]] || usage
[[ "$version" =~ ^[0-9][0-9A-Za-z.+~-]*$ ]] || {
  echo "invalid bundle version: $version" >&2
  exit 2
}

[[ -x "$binary_dir/music-player-desktop" ]] || {
  echo "missing executable: $binary_dir/music-player-desktop" >&2
  exit 1
}

app_name="Music Player"
bundle_id="com.github.tsirysndr.music-player"
build_root="$script_dir/build"
app="$build_root/macos/$app_name.app"
dmg="$build_root/$app_name.dmg"

rm -rf "$build_root"
install -d "$app/Contents/MacOS" "$app/Contents/Resources"
install -m 0755 "$binary_dir/music-player-desktop" "$app/Contents/MacOS/music-player-desktop"
install -m 0644 "$repo_root/desktop/assets/AppIcon.icns" "$app/Contents/Resources/AppIcon.icns"

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>$app_name</string>
  <key>CFBundleExecutable</key>
  <string>music-player-desktop</string>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
  <key>CFBundleIdentifier</key>
  <string>$bundle_id</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>$app_name</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>$version</string>
  <key>CFBundleVersion</key>
  <string>$version</string>
  <key>LSApplicationCategoryType</key>
  <string>public.app-category.music</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key>
  <true/>
</dict>
</plist>
PLIST

# Ad-hoc signature. This is NOT notarization -- a downloaded build still
# carries the quarantine bit and needs right-click > Open (or `xattr -cr`).
# But without it the bundle ships only a linker-signed Mach-O with no sealed
# resources, and Gatekeeper refuses it outright: "code has no resources but
# signature indicates they must be present". Export MACOS_SIGNING_IDENTITY to
# sign with a real Developer ID instead.
signing_identity=${MACOS_SIGNING_IDENTITY:--}
codesign --force --sign "$signing_identity" --identifier "$bundle_id" \
  --timestamp=none "$app"
codesign --verify --strict --verbose=2 "$app"

build_dmg() {
  local dmg_root="$build_root/dmg"

  rm -rf "$dmg_root"
  install -d "$dmg_root"
  cp -R "$app" "$dmg_root/$app_name.app"
  ln -s /Applications "$dmg_root/Applications"

  rm -f "$dmg"
  hdiutil create -volname "$app_name" -srcfolder "$dmg_root" \
    -ov -format UDZO "$dmg" >/dev/null
  codesign --force --sign "$signing_identity" --timestamp=none "$dmg"
}

case "$format" in
  app) ;;
  dmg) build_dmg ;;
  all) build_dmg ;;
esac
