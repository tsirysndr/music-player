module Dagger.Errors (DaggerError(..), GraphQLError, ExecError, fromError, tryDagger) where

import Prelude
import Data.Argonaut.Core (Json)
import Data.Array (null)
import Data.Either (Either(..))
import Data.Maybe (Maybe(..))
import Data.Nullable (Nullable, toMaybe)
import Effect.Aff (Aff, try)
import Effect.Exception (Error)

type GraphQLError = { message :: String, path :: Json, extensions :: Json }
type ExecError = { message :: String, cmd :: Array String, exitCode :: Int, stdout :: String, stderr :: String }

data DaggerError
  = ExecFailure ExecError
  | GraphQLFailure (Array GraphQLError)
  | RequestFailure String

foreign import inspectError :: Error -> { message :: String, errors :: Array GraphQLError, exec :: Nullable ExecError }

fromError :: Error -> DaggerError
fromError err =
  let info = inspectError err
  in case toMaybe info.exec of
    Just exec -> ExecFailure exec
    Nothing -> if null info.errors then RequestFailure info.message else GraphQLFailure info.errors

tryDagger :: forall a. Aff a -> Aff (Either DaggerError a)
tryDagger action = do
  result <- try action
  pure case result of
    Left err -> Left (fromError err)
    Right value -> Right value
