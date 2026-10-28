module Release.Common where

import Prelude
import Data.Array as A
import Data.String as S
import Dagger (Client)
import Dagger.Container as C
import Dagger.Query as Q
import Dagger.Types (Container)
import Effect.Aff (Aff, throwError)
import Effect.Exception (error)

ensure :: Boolean -> String -> Aff Unit
ensure condition message = unless condition (throwError (error message))

filename :: String -> Aff Unit
filename name = ensure (name /= "" && name /= "." && name /= ".." && not (S.contains (S.Pattern "/") name) && not (S.contains (S.Pattern "\n") name) && not (S.contains (S.Pattern "\r") name)) "Expected a nonempty filename, without directories or line breaks"

relative :: String -> Aff Unit
relative path = ensure (path /= "" && S.take 1 path /= "/" && not (A.elem ".." (S.split (S.Pattern "/") path))) "Expected a relative path without parent traversal"

base :: Client -> Container
base client = Q.container client # C.from "ubuntu:24.04" # C.withWorkdir "/work"

apt :: Array String -> Container -> Container
apt packages = C.withExec [ "apt-get", "update" ] >>> C.withExec ([ "apt-get", "install", "-y", "--no-install-recommends" ] <> packages)
