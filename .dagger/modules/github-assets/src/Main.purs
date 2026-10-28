module Main where

import Prelude
import Dagger.Modules as Module
import Effect (Effect)
import Release.GithubAssets as Release

main :: Effect Unit
main = Module.runModule
  [ Module.object_ "GithubAssets" {}
      [ Module.function "upload" "Upload assets to an existing GitHub release."
          (ado
            assets <- Module.argument "assets"
            files <- Module.argument "files"
            repository <- Module.argument "repository"
            tag <- Module.argument "tag"
            token <- Module.argument "token"
            runId <- Module.argument "runId"
            clobber <- Module.defaultArgument "clobber" false
            in { runId, assets, files, repository, tag, token, clobber }) Release.upload
      ]
  ]
