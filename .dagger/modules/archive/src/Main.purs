module Main where

import Prelude
import Dagger.Modules as Module
import Effect (Effect)
import Release.TarGz as TarGz
import Release.Sha256 as Sha256

main :: Effect Unit
main = Module.runModule
  [ Module.object_ "Archive" {}
      [ Module.function "tarGz" "Create a gzip compressed tar archive."
          (ado
            source <- Module.argument "source"
            name <- Module.argument "name"
            members <- Module.defaultArgument "members" ([ "." ] :: Array String)
            in { source, name, members }) TarGz.create
      , Module.function "sha256" "Generate a sha256sum compatible sidecar file."
          (ado
            file <- Module.argument "file"
            name <- Module.argument "name"
            in { file, name }) Sha256.generate
      ]
  ]
