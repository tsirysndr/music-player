module Release.Deb where

import Prelude
import Release.Common as Common
import Dagger (Client)
import Dagger.Container as C
import Dagger.Types (Container, Directory, File)
import Effect.Aff (Aff)

buildIn :: String -> String -> Container -> Container
buildIn root output = C.withExec [ "dpkg-deb", "--build", "--root-owner-group", root, output ]

build :: Client -> { root :: Directory, name :: String } -> Aff File
build client { root, name } = do
  Common.filename name
  pure $ Common.base client # Common.apt [ "dpkg-dev" ]
    # C.withDirectory "/package" root # buildIn "/package" ("/work/" <> name)
    # C.file ("/work/" <> name)
