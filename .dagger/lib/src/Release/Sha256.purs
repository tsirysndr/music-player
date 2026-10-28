module Release.Sha256 where

import Prelude
import Release.Common as Common
import Data.Maybe (Maybe(..))
import Dagger (Client)
import Dagger.Container as C
import Dagger.Types (Container, File)
import Effect.Aff (Aff)

checksum :: String -> Container -> Container
checksum path = C.withExecWith [ "sha256sum", path ] (C.defaultWithExecOpts { redirectStdout = Just (path <> ".sha256") })

generate :: Client -> { file :: File, name :: String } -> Aff File
generate client { file, name } = do
  Common.filename name
  pure $ Common.base client # C.withFile ("/work/" <> name) file
    # checksum ("./" <> name) # C.file (name <> ".sha256")
