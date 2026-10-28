module Main where

import Prelude
import Dagger.Modules as Module
import Effect (Effect)
import Release.Gemfury as Release

main :: Effect Unit
main = Module.runModule
  [ Module.object_ "Gemfury" {}
      [ Module.function "upload" "Upload package files to Gemfury."
          (ado
            packages <- Module.argument "packages"
            files <- Module.argument "files"
            account <- Module.argument "account"
            token <- Module.argument "token"
            runId <- Module.argument "runId"
            in { runId, packages, files, account, token }) Release.upload
      ]
  ]
