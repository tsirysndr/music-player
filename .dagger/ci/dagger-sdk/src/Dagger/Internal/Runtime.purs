module Dagger.Internal.Runtime
  ( Session, Object, Argument, connect, connectWith, defaultConfig, Config
  , root, sessionOf, select, fragment, argument, objectArgument, objectsArgument
  , optional, scalar, objectList, nullableObject, execute, rawQuery
  ) where

import Prelude
import Data.Argonaut.Core (Json)
import Data.Argonaut.Decode (class DecodeJson, decodeJson)
import Data.Argonaut.Encode (class EncodeJson, encodeJson)
import Data.Array (catMaybes)
import Data.Either (Either(..))
import Data.Maybe (Maybe(..))
import Data.Traversable (traverse)
import Effect (Effect)
import Effect.Aff (Aff, Canceler(..), makeAff, throwError)
import Effect.Class (liftEffect)
import Effect.Exception (Error, error)

foreign import data Session :: Type
newtype Object :: Type -> Type
newtype Object a = Object { session :: Session, steps :: Array Step }
type role Object nominal
type Step = { name :: String, args :: Array Argument }
type ResolvedStep = { name :: String, args :: Array { name :: String, typeName :: String, value :: Json } }
newtype Argument = Argument { name :: String, typeName :: String, value :: Aff Json }
type Config = { port :: Int, token :: String, timeoutMs :: Int }

defaultConfig :: Config
defaultConfig = { port: 0, token: "", timeoutMs: 300000 }

foreign import connect :: Effect Session
foreign import connectWith :: Config -> Effect Session
foreign import requestImpl :: Session -> Array ResolvedStep -> (Error -> Effect Unit) -> (Json -> Effect Unit) -> Effect (Effect Unit)
foreign import rawImpl :: Session -> String -> Json -> (Error -> Effect Unit) -> (Json -> Effect Unit) -> Effect (Effect Unit)

root :: forall a. Session -> Object a
root session = Object { session, steps: [] }

select :: forall a b. String -> Array (Maybe Argument) -> Object a -> Object b
select name args (Object o) = Object (o { steps = o.steps <> [{ name, args: catMaybes args }] })

fragment :: forall a b. String -> Object a -> Object b
fragment name = select ("... on " <> name) []

argument :: forall a. EncodeJson a => String -> String -> a -> Maybe Argument
argument name typeName value = Just (Argument { name, typeName, value: pure (encodeJson value) })

optional :: forall a. (a -> Maybe Argument) -> Maybe a -> Maybe Argument
optional f = case _ of
  Nothing -> Nothing
  Just a -> f a

objectArgument :: forall a. String -> String -> Object a -> Maybe Argument
objectArgument name typeName obj = Just (Argument { name, typeName, value: execute (select "id" [] obj) })

objectsArgument :: forall a. String -> String -> Array (Object a) -> Maybe Argument
objectsArgument name typeName objs = Just (Argument { name, typeName, value: encodeJson <$> traverse (scalar "id" [] :: Object a -> Aff String) objs })

execute :: forall a. Object a -> Aff Json
execute (Object o) = do
  steps <- traverse (\s -> do
    args <- traverse (\(Argument a) -> do
      value <- a.value
      pure { name: a.name, typeName: a.typeName, value }) s.args
    pure { name: s.name, args }) o.steps
  makeAff \done -> do
    cancel <- requestImpl o.session steps (done <<< Left) (done <<< Right)
    pure (Canceler (\_ -> liftEffect cancel))

scalar :: forall a b. DecodeJson b => String -> Array (Maybe Argument) -> Object a -> Aff b
scalar name args obj = do
  value <- execute (select name args obj)
  case decodeJson value of
    Left err -> throwError (error ("Dagger response decoding failed: " <> show err))
    Right result -> pure result

objectList :: forall a b. String -> String -> Array (Maybe Argument) -> Object a -> Aff (Array (Object b))
objectList typeName name args obj@(Object o) = do
  ids <- scalar "id" [] (select name args obj) :: Aff (Array String)
  pure (map (\id -> fragment typeName (select "node" [argument "id" "ID!" id] (root o.session))) ids)

nullableObject :: forall a b. String -> String -> Array (Maybe Argument) -> Object a -> Aff (Maybe (Object b))
nullableObject typeName name args obj@(Object o) = do
  id <- scalar "id" [] (select name args obj) :: Aff (Maybe String)
  pure (map (\value -> fragment typeName (select "node" [argument "id" "ID!" value] (root o.session))) id)

rawQuery :: Session -> String -> Json -> Aff Json
rawQuery session query variables = makeAff \done -> do
  cancel <- rawImpl session query variables (done <<< Left) (done <<< Right)
  pure (Canceler (\_ -> liftEffect cancel))

sessionOf :: forall a. Object a -> Session
sessionOf (Object o) = o.session
