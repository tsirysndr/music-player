module CI.Package (linux, mac) where

import Prelude
import CI.Support as S
import Data.Array as A
import Data.Foldable (traverse_)
import Data.String as String
import Effect (Effect)

replace :: String -> String -> String -> String
replace needle value = String.replaceAll (String.Pattern needle) (String.Replacement value)

-- Staging always uses a fresh temporary directory, so no earlier package files
-- can be included accidentally and the release directory holds assets only.
linux :: String -> String -> String -> String -> Effect Unit
linux binaryDir version arch out = do
  temporary <- String.trim <$> S.captureIn "." [ "mktemp", "-d" ]
  let root = temporary <> "/root"
      bare = if String.take 1 version == "v" then String.drop 1 version else version
      debArch = if arch == "x86_64" then "amd64" else "arm64"
      deb = "music-player_" <> bare <> "_" <> debArch <> ".deb"
      rpm = temporary <> "/rpmbuild"
  S.run [ "mkdir", "-p", out <> "/dist", root <> "/usr/bin", root <> "/usr/share/applications", root <> "/usr/share/icons/hicolor/scalable/apps", root <> "/usr/share/licenses/music-player", root <> "/DEBIAN" ]
  S.run [ "install", "-m", "0755", binaryDir <> "/music-player", binaryDir <> "/music-player-desktop", root <> "/usr/bin/" ]
  S.run [ "install", "-m", "0644", "dist/music-player.desktop", root <> "/usr/share/applications/music-player.desktop" ]
  S.run [ "install", "-m", "0644", "desktop/assets/icon.svg", root <> "/usr/share/icons/hicolor/scalable/apps/music-player.svg" ]
  S.run [ "install", "-m", "0644", "LICENSE", root <> "/usr/share/licenses/music-player/LICENSE" ]
  control <- S.readText "dist/debian-control.in"
  S.writeText (root <> "/DEBIAN/control") (control # replace "@VERSION@" bare # replace "@DEB_ARCH@" debArch)
  S.run [ "dpkg-deb", "--build", "--root-owner-group", root, out <> "/dist/" <> deb ]
  S.checksum (out <> "/dist/" <> deb)
  S.run ([ "mkdir", "-p" ] <> map (\name -> rpm <> "/" <> name) [ "BUILD", "BUILDROOT", "RPMS", "SOURCES", "SPECS", "SRPMS" ])
  S.run [ "mkdir", "-p", rpm <> "/SOURCES/root" ]
  S.run [ "cp", "-R", root <> "/usr", rpm <> "/SOURCES/root/" ]
  spec <- S.readText "dist/music-player.spec.in"
  S.writeText (rpm <> "/SPECS/music-player.spec") (spec # replace "@VERSION@" (replace "-" "~" bare) # replace "@RPM_ARCH@" arch)
  S.run [ "rpmbuild", "--define", "_topdir " <> rpm, "--target", arch, "-bb", rpm <> "/SPECS/music-player.spec" ]
  packages <- A.filter (String.endsWith (String.Pattern ".rpm")) <$> S.files (rpm <> "/RPMS")
  traverse_ (\path -> do
    let destination = out <> "/dist/" <> S.basename path
    S.run [ "cp", path, destination ]
    S.checksum destination) packages

plist :: String -> String
plist version = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleDevelopmentRegion</key><string>en</string>
<key>CFBundleDisplayName</key><string>Music Player</string>
<key>CFBundleExecutable</key><string>music-player-desktop</string>
<key>CFBundleIconFile</key><string>AppIcon</string>
<key>CFBundleIdentifier</key><string>com.github.tsirysndr.music-player</string>
<key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
<key>CFBundleName</key><string>Music Player</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>""" <> version <> """</string>
<key>CFBundleVersion</key><string>""" <> version <> """</string>
<key>LSApplicationCategoryType</key><string>public.app-category.music</string>
<key>LSMinimumSystemVersion</key><string>11.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSSupportsAutomaticGraphicsSwitching</key><true/>
</dict></plist>
"""

mac :: String -> String -> String -> String -> String -> Effect Unit
mac binaryDir version target asset out = do
  temporary <- String.trim <$> S.captureIn "." [ "mktemp", "-d" ]
  let app = temporary <> "/macos/Music Player.app"
      bare = if String.take 1 version == "v" then String.drop 1 version else version
      dmg = "Music_Player_" <> version <> "_" <> (if target == "aarch64-apple-darwin" then "aarch64" else "x64") <> ".dmg"
      dmgRoot = temporary <> "/dmg"
  identity <- S.env "MACOS_SIGNING_IDENTITY" "-"
  S.run [ "mkdir", "-p", app <> "/Contents/MacOS", app <> "/Contents/Resources", dmgRoot ]
  S.run [ "install", "-m", "0755", binaryDir <> "/music-player-desktop", app <> "/Contents/MacOS/music-player-desktop" ]
  S.run [ "install", "-m", "0644", "desktop/assets/AppIcon.icns", app <> "/Contents/Resources/AppIcon.icns" ]
  S.writeText (app <> "/Contents/Info.plist") (plist bare)
  S.run [ "codesign", "--force", "--sign", identity, "--identifier", "com.github.tsirysndr.music-player", "--timestamp=none", app ]
  S.run [ "codesign", "--verify", "--strict", "--verbose=2", app ]
  S.run [ "env", "COPYFILE_DISABLE=1", "tar", "czf", out <> "/" <> asset, "-C", temporary, "macos/Music Player.app" ]
  S.run [ "cp", "-R", app, dmgRoot <> "/Music Player.app" ]
  S.run [ "ln", "-s", "/Applications", dmgRoot <> "/Applications" ]
  S.run [ "hdiutil", "create", "-volname", "Music Player", "-srcfolder", dmgRoot, "-ov", "-format", "UDZO", out <> "/" <> dmg ]
  S.run [ "codesign", "--force", "--sign", identity, "--timestamp=none", out <> "/" <> dmg ]
  S.checksum (out <> "/" <> dmg)
