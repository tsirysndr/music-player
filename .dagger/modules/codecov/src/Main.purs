module Main where

import Prelude
import Dagger.Modules as Module
import Effect (Effect)
import Release.Codecov as Codecov

main :: Effect Unit
main = Module.runModule
  [ Module.object_ "Codecov" {}
      [ Module.function "upload" "Upload a coverage report to Codecov with explicit commit metadata."
          (ado
            report <- Module.argument "report"
            repository <- Module.argument "repository"
            commit <- Module.argument "commit"
            branch <- Module.argument "branch"
            token <- Module.argument "token"
            runId <- Module.argument "runId"
            pullRequest <- Module.defaultArgument "pullRequest" ""
            flags <- Module.defaultArgument "flags" []
            in { report, repository, commit, branch, token, runId, pullRequest, flags }) Codecov.upload
      ]
  ]
