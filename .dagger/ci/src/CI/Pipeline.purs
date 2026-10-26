module CI.Pipeline where

import Prelude
import CI.Integration as Integration
import CI.Release as Release
import Dagger (Client)
import Dagger.Directory as Directory
import Dagger.File as File
import Dagger.Types (Directory)
import Effect.Aff (Aff)

type BuildInput = { source :: Directory, target :: String, tag :: String }

build :: Client -> BuildInput -> Aff Directory
build client input = Release.linux client
  { source: input.source, target: input.target, version: input.tag, desktop: false }

type Input = { source :: Directory, platform :: String, target :: String, tag :: String }

run :: Client -> Input -> Aff Directory
run client input = do
  coverage <- Integration.test client { source: input.source, platform: input.platform }
  -- Finish tests before starting the release build, as the original runner did.
  void (File.sync coverage)
  artifacts <- build client { source: input.source, target: input.target, tag: input.tag }
  pure (artifacts # Directory.withFile "lcov.info" coverage)
