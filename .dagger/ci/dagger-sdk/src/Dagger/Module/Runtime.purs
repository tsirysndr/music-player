module Dagger.Module.Runtime (runModule, serve, register, invoke, Call) where

import Prelude
import Data.Argonaut.Core (Json, stringify)
import Data.Argonaut.Encode (encodeJson)
import Data.Argonaut.Parser (jsonParser)
import Data.Array (find, length, nub)
import Data.Either (Either(..))
import Data.Foldable (foldl, traverse_)
import Data.Maybe (Maybe(..))
import Data.Traversable (traverse)
import Data.Tuple (Tuple(..))
import Dagger (Client, withConnection)
import Dagger.Function as Function
import Dagger.FunctionCall as FunctionCall
import Dagger.FunctionCallArgValue as CallArg
import Dagger.Module as EngineModule
import Dagger.Module.Definition (ObjectDefinition, invokeObject, objectFunctions, objectName)
import Dagger.Module.Value (ValueType(..))
import Dagger.Query as Query
import Dagger.TypeDef as TypeDef
import Dagger.Types as T
import Effect (Effect)
import Effect.Aff (Aff, launchAff_, throwError, try)
import Effect.Exception (error, message)
import Foreign.Object as Object

type Call = { parentName :: String, name :: String, parent :: Json, args :: Object.Object Json }

runModule :: Array ObjectDefinition -> Effect Unit
runModule definitions = launchAff_ $ withConnection \client -> serve client definitions

serve :: Client -> Array ObjectDefinition -> Aff Unit
serve client definitions = do
  let call = Query.currentFunctionCall client
  outcome <- try do
    validate definitions
    parentName <- FunctionCall.parentName call
    if parentName == "" then encodeJson <$> register client definitions
    else do
      name <- FunctionCall.name call
      T.JSON parent <- FunctionCall.parent call
      parsedParent <- parse parent
      args <- FunctionCall.inputArgs call >>= traverse (\arg -> do
        argName <- CallArg.name arg
        T.JSON value <- CallArg.value arg
        parsed <- parse value
        pure (Tuple argName parsed))
      invoke client definitions { parentName, name, parent: parsedParent, args: Object.fromFoldable args }
  case outcome of
    Right result -> void $ FunctionCall.returnValue (T.JSON (stringify result)) call
    Left err -> do
      void $ FunctionCall.returnError (Query.error (message err) client) call
      throwError err

register :: Client -> Array ObjectDefinition -> Aff T.ID
register client definitions = do
  validate definitions
  EngineModule.id $ foldl (\mod obj -> EngineModule.withObject (objectType obj) mod) (Query.module_ client) definitions
  where
  objectType obj = foldl addFunction (TypeDef.withObject (objectName obj) (Query.typeDef client)) (objectFunctions obj)
    where
    addFunction owner fn =
      let
        result = case fn.returnType of
          Nothing -> TypeDef.withObject (objectName obj) (Query.typeDef client)
          Just t -> toTypeDef client t
        definition = foldl (\f arg -> Function.withArgWith arg.name (toTypeDef client arg.valueType)
          (Function.defaultWithArgOpts { defaultValue = map (T.JSON <<< stringify) arg.defaultValue }) f)
          (Query.function fn.name result client # Function.withDescription fn.description) fn.arguments
      in if fn.name == "" then TypeDef.withConstructor definition owner else TypeDef.withFunction definition owner

invoke :: Client -> Array ObjectDefinition -> Call -> Aff Json
invoke client definitions call = case find (\obj -> objectName obj == call.parentName) definitions of
  Nothing -> throwError (error ("Unknown module object " <> call.parentName))
  Just obj -> invokeObject client call.name call.parent call.args obj

toTypeDef :: Client -> ValueType -> T.TypeDef
toTypeDef client value = case value of
  StringType -> kind T.TypeDefKindStringKind
  IntType -> kind T.TypeDefKindIntegerKind
  NumberType -> kind T.TypeDefKindFloatKind
  BooleanType -> kind T.TypeDefKindBooleanKind
  VoidType -> kind T.TypeDefKindVoidKind # TypeDef.withOptional true
  ListType t -> base # TypeDef.withListOf (toTypeDef client t)
  OptionalType t -> toTypeDef client t # TypeDef.withOptional true
  ObjectType name -> base # TypeDef.withObject name
  EnumType name -> base # TypeDef.withEnum name
  ScalarType name -> base # TypeDef.withScalar name
  where
  base = Query.typeDef client
  kind t = base # TypeDef.withKind t

validate :: Array ObjectDefinition -> Aff Unit
validate objects = do
  unique "object" (map objectName objects)
  traverse_ (\obj -> do
    let functions = objectFunctions obj
    unique (objectName obj <> " function") (map _.name functions)
    traverse_ (\f -> unique (f.name <> " argument") (map _.name f.arguments)) functions) objects
  where
  unique label names = unless (length names == length (nub names)) $ throwError (error ("Duplicate " <> label <> " name"))

parse :: String -> Aff Json
parse text = case jsonParser text of
  Left err -> throwError (error ("Invalid module JSON: " <> err))
  Right value -> pure value
