module Release.Gemfury where

import Prelude
import Release.Common as Common
import Data.Foldable (all, foldl, traverse_)
import Data.String.CodeUnits as CU
import Data.String as S
import Dagger (Client)
import Dagger.Container as C
import Dagger.Types (Container, Directory, Secret)
import Effect.Aff (Aff)

type Options = { packages :: Directory, files :: Array String, account :: String, token :: Secret, runId :: String }

uploadGraph :: Client -> Options -> Aff Container
uploadGraph client opts = do
  Common.ensure (opts.files /= [] && opts.runId /= "") "Package files and a unique upload run ID are required"
  Common.ensure (opts.account /= "") "A Gemfury account is required"
  -- Account is a single URL path segment, never a host or query string.
  Common.ensure (all (\c -> (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '-' || c == '_') (CU.toCharArray opts.account)) "Invalid Gemfury account"
  traverse_ Common.relative opts.files
  let base = Common.base client # Common.apt [ "curl", "ca-certificates" ]
        # C.withEnvVariable "UPLOAD_RUN_ID" opts.runId
        # C.withDirectory "/packages" opts.packages # C.withSecretVariable "FURY_TOKEN" opts.token
  pure $ foldl (\ctr path -> ctr # C.withExec
    [ "curl", "--fail-with-body", "--silent", "--show-error", "--variable", "%FURY_TOKEN", "--expand-user", "{{FURY_TOKEN}}:", "--form", "package=@\"/packages/" <> formPath path <> "\"", "https://push.fury.io/" <> opts.account <> "/" ]) base opts.files

upload :: Client -> Options -> Aff String
upload client opts = do
  graph <- uploadGraph client opts
  _ <- C.sync graph
  pure "Gemfury packages uploaded"

-- curl's multipart grammar treats commas and semicolons specially even without a shell.
formPath :: String -> String
formPath = S.replaceAll (S.Pattern "\\") (S.Replacement "\\\\")
  >>> S.replaceAll (S.Pattern "\"") (S.Replacement "\\\"")
