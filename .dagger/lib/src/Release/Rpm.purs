module Release.Rpm where

import Prelude
import Release.Common as Common
import Data.Foldable (foldl)
import Data.String as S
import Data.Maybe (Maybe(..))
import Dagger (Client)
import Dagger.Container as C
import Dagger.Directory as D
import Dagger.Query as Q
import Dagger.Types (Container, Directory, File, Platform(..))
import Effect.Aff (Aff)

buildIn :: String -> String -> Container -> Container
buildIn spec target = C.withExec [ "mkdir", "-p", "/tmp/rpmbuild/BUILD", "/tmp/rpmbuild/BUILDROOT", "/tmp/rpmbuild/RPMS", "/tmp/rpmbuild/SOURCES", "/tmp/rpmbuild/SPECS", "/tmp/rpmbuild/SRPMS" ]
  >>> C.withExec [ "rpmbuild", "--define", "_topdir /tmp/rpmbuild", "--target", target, "-bb", spec ]

basename :: String -> String
basename path = foldl (\_ part -> part) "" (S.split (S.Pattern "/") path)

build :: Client -> { sources :: Directory, spec :: File, target :: String } -> Aff Directory
build client { sources, spec, target } = do
  Common.ensure (target /= "" && S.take 1 target /= "-") "An RPM target architecture is required"
  -- RPM post-processing (including strip) must run on the package architecture.
  let platform = case target of
        "x86_64" -> Just (Platform "linux/amd64")
        "aarch64" -> Just (Platform "linux/arm64")
        _ -> Nothing
      built = Q.containerWith (Q.defaultContainerOpts { platform = platform }) client
        # C.from "ubuntu:24.04" # C.withWorkdir "/work"
        # Common.apt [ "rpm", "build-essential" ]
        # C.withDirectory "/tmp/rpmbuild/SOURCES" sources # C.withFile "/package.spec" spec
        # buildIn "/package.spec" target
  paths <- D.glob "**/*.rpm" (C.directory "/tmp/rpmbuild/RPMS" built)
  Common.ensure (paths /= []) "rpmbuild produced no packages"
  pure $ foldl (\out path -> D.withFile path (C.file ("/tmp/rpmbuild/RPMS/" <> path) built) out) (Q.directory client) paths
