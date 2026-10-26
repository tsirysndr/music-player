module CI.Web where

import Prelude
import CI.Base as Base
import Dagger (Client)
import Dagger.Container as Container
import Dagger.Directory as Directory
import Dagger.Query as Query
import Dagger.Types (Container, Directory, Platform)
import Effect.Aff (Aff)

prepared :: Client -> Platform -> Directory -> Container
prepared client arch source = Base.container client arch "node:24-bookworm-slim"
  # Container.withExec [ "npm", "install", "--global", "bun@1.4.3" ]
  # Container.withMountedCache "/root/.bun/install/cache" (Query.cacheVolume "music-player-bun-1.4.3" client)
  # Container.withDirectory "/src" (Base.sourceTree client source)
  # Container.withWorkdir "/src/webui/musicplayer"
  # Container.withExec [ "bun", "install", "--frozen-lockfile" ]

build :: Client -> Platform -> Directory -> Directory
build client arch source = prepared client arch source
  # Container.withExec [ "bun", "run", "build" ]
  # Container.directory "/src/webui/musicplayer/build"

type Input = { source :: Directory, platform :: String }

check :: Client -> Input -> Aff Directory
check client input = do
  arch <- Base.platform input.platform
  let
    checked = prepared client arch input.source
      # Container.withExec [ "bunx", "tsc", "--noEmit" ]
      # Container.withExec [ "bun", "run", "test:coverage" ]
      # Container.withExec [ "bunx", "vite", "build" ]
      # Container.withExec [ "bunx", "storybook", "build" ]
  pure $ Query.directory client
    # Directory.withDirectory "coverage" (Container.directory "/src/webui/musicplayer/coverage" checked)
    # Directory.withDirectory "build" (Container.directory "/src/webui/musicplayer/build" checked)
    # Directory.withDirectory "storybook-static" (Container.directory "/src/webui/musicplayer/storybook-static" checked)
