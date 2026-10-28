module Release.GithubAssets where

import Prelude
import Release.Common as Common
import Data.String as S
import Data.Foldable (traverse_)
import Dagger (Client)
import Dagger.Container as C
import Dagger.Types (Container, Directory, Secret)
import Effect.Aff (Aff)

type Options = { assets :: Directory, files :: Array String, repository :: String, tag :: String, token :: Secret, clobber :: Boolean, runId :: String }

uploadGraph :: Client -> Options -> Aff Container
uploadGraph client opts = do
  Common.ensure (opts.runId /= "" && opts.files /= [] && opts.repository /= "" && opts.tag /= "") "Files, repository, release tag and a unique upload run ID are required"
  traverse_ Common.relative opts.files
  traverse_ (\path -> Common.ensure (not (S.contains (S.Pattern "#") path)) "GitHub asset paths cannot contain # (reserved for labels)") opts.files
  pure $ Common.base client # Common.apt [ "gh", "ca-certificates" ]
    # C.withEnvVariable "UPLOAD_RUN_ID" opts.runId
    # C.withDirectory "/assets" opts.assets # C.withWorkdir "/assets"
    # C.withSecretVariable "GH_TOKEN" opts.token
    # C.withExec ([ "gh", "release", "upload", "--repo", opts.repository ]
        <> (if opts.clobber then [ "--clobber" ] else [])
        <> [ "--", opts.tag ] <> map ("/assets/" <> _) opts.files)

upload :: Client -> Options -> Aff String
upload client opts = do
  graph <- uploadGraph client opts
  _ <- C.sync graph
  pure "GitHub release assets uploaded"
