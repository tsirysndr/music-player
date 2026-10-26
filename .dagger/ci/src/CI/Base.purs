module CI.Base where

import Prelude
import Data.Maybe (Maybe(..))
import Dagger (Client)
import Dagger.Container as Container
import Dagger.Directory as Directory
import Dagger.Query as Query
import Dagger.Types (Container, Directory, Platform(..))
import Effect.Aff (Aff, throwError)
import Effect.Exception (error)

-- Keep generated CI code, host caches, and secrets out of build inputs.
sourceTree :: Client -> Directory -> Directory
sourceTree client source = Query.directory client
  # Directory.withDirectoryWith "." source
      (Directory.defaultWithDirectoryOpts
        { exclude = Just [ ".git", ".devbox", ".dagger", ".github", "target", "**/node_modules", "**/.env", "**/.env.*", "artifacts", "release-source", "vendor", "dist/build", "dist/rpmbuild", "result" ] })

platform :: String -> Aff Platform
platform value = case value of
  "linux/amd64" -> pure (Platform value)
  "linux/arm64" -> pure (Platform value)
  _ -> throwError (error "Supported platforms: linux/amd64 and linux/arm64")

container :: Client -> Platform -> String -> Container
container client arch image = Query.containerWith (Query.defaultContainerOpts { platform = Just arch }) client
  # Container.from image

shell :: String -> Container -> Container
shell script = Container.withExec [ "bash", "-euo", "pipefail", "-c", script ]
