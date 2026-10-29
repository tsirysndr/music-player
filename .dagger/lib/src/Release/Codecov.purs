module Release.Codecov where

import Prelude
import Data.Array (concatMap)
import Dagger (Client)
import Dagger.Container as C
import Dagger.Query as Q
import Dagger.Types (Container, File, Secret)
import Effect.Aff (Aff)
import Release.Common as Common

type Options =
  { report :: File, repository :: String, commit :: String, branch :: String
  , token :: Secret, runId :: String, pullRequest :: String, flags :: Array String
  }

uploadGraph :: Client -> Options -> Aff Container
uploadGraph client opts = do
  Common.ensure (opts.repository /= "" && opts.commit /= "" && opts.branch /= "" && opts.runId /= "")
    "Repository, commit, branch and a unique upload run ID are required"
  pure $ Q.container client # C.from "python:3.12-slim-bookworm"
    # Common.apt [ "git", "ca-certificates" ]
    # C.withExec [ "pip", "install", "--no-cache-dir", "codecov-cli==11.3.2" ]
    # C.withWorkdir "/src"
    # C.withFile "/coverage/lcov.info" opts.report
    # C.withEnvVariable "UPLOAD_RUN_ID" opts.runId
    # C.withSecretVariable "CODECOV_TOKEN" opts.token
    # C.withExec
        ([ "codecovcli", "upload-process", "--disable-search", "--disable-file-fixes"
         , "--plugin", "noop", "--fail-on-error", "--git-service", "github"
         , "--file", "/coverage/lcov.info", "--slug", opts.repository
         , "--sha", opts.commit, "--branch", opts.branch, "--name", opts.runId
         ]
          <> (if opts.pullRequest == "" then [] else [ "--pr", opts.pullRequest ])
          <> concatMap (\flag -> [ "--flag", flag ]) opts.flags)

upload :: Client -> Options -> Aff String
upload client opts = do
  graph <- uploadGraph client opts
  _ <- C.sync graph
  pure "Codecov coverage uploaded"
