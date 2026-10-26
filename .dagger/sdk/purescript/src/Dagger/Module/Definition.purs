-- | Typed module definitions. Argument types and return types are inferred from
-- | handlers. Applicative arguments compose naturally with `ado` notation.
module Dagger.Module.Definition
  ( Arguments, ArgumentDefinition, ModuleFunction, ObjectDefinition
  , argument, defaultArgument, function, method, constructor, stateful
  , object_, objectName, objectFunctions, functionDefinition, invokeObject
  ) where

import Prelude
import Data.Argonaut.Core (Json, jsonNull)
import Data.Argonaut.Decode (class DecodeJson)
import Data.Argonaut.Encode (class EncodeJson, encodeJson)
import Data.Array (find, any)
import Data.Either (Either(..))
import Data.Maybe (Maybe(..), fromMaybe)
import Dagger (Client)
import Dagger.Internal.Runtime (Session, sessionOf)
import Dagger.Module.Value (class DaggerValue, ValueType, decodeJSON, decodeValue, encodeValue, valueType)
import Effect.Aff (Aff, throwError)
import Effect.Exception (error)
import Foreign.Object as Object
import Type.Proxy (Proxy(..))

type ArgumentDefinition =
  { name :: String, valueType :: ValueType, defaultValue :: Maybe Json }

newtype Arguments a = Arguments
  { definitions :: Array ArgumentDefinition
  , decode :: Session -> Object.Object Json -> Either String a
  }

instance functorArguments :: Functor Arguments where
  map f (Arguments a) = Arguments { definitions: a.definitions, decode: \s values -> f <$> a.decode s values }

instance applyArguments :: Apply Arguments where
  apply (Arguments f) (Arguments a) = Arguments
    { definitions: f.definitions <> a.definitions
    , decode: \s values -> f.decode s values <*> a.decode s values
    }

instance applicativeArguments :: Applicative Arguments where
  pure a = Arguments { definitions: [], decode: \_ _ -> Right a }

argument :: forall a. DaggerValue a => String -> Arguments a
argument name = argumentImpl name Nothing

defaultArgument :: forall a. DaggerValue a => EncodeJson a => String -> a -> Arguments a
defaultArgument name value = argumentImpl name (Just (encodeJson value))

argumentImpl :: forall a. DaggerValue a => String -> Maybe Json -> Arguments a
argumentImpl name defaultValue = Arguments
  { definitions: [{ name, valueType: valueType (Proxy :: Proxy a), defaultValue }]
  , decode: \session values -> case decodeValue session (fromMaybe (fromMaybe jsonNull defaultValue) (Object.lookup name values)) of
      Left message -> Left ("Argument " <> name <> ": " <> message)
      Right result -> Right result
  }

type FunctionDefinition =
  { name :: String, description :: String, arguments :: Array ArgumentDefinition, returnType :: Maybe ValueType }

newtype ModuleFunction state = ModuleFunction
  { definition :: FunctionDefinition
  , invoke :: Client -> state -> Object.Object Json -> Aff Json
  }

function :: forall state a b. DaggerValue b => String -> String -> Arguments a -> (Client -> a -> Aff b) -> ModuleFunction state
function name description args handler = method name description args (\client _ -> handler client)

method :: forall state a b. DaggerValue b => String -> String -> Arguments a -> (Client -> state -> a -> Aff b) -> ModuleFunction state
method name description (Arguments args) handler = ModuleFunction
  { definition: { name, description, arguments: args.definitions, returnType: Just (valueType (Proxy :: Proxy b)) }
  , invoke: \client state values -> do
      decoded <- orThrow (args.decode (sessionOf client) values)
      handler client state decoded >>= encodeValue
  }

-- | A state transition returns the new object value; callers may chain methods.
stateful :: forall state a. EncodeJson state => String -> String -> Arguments a -> (Client -> state -> a -> Aff state) -> ModuleFunction state
stateful name description (Arguments args) handler = ModuleFunction
  { definition: { name, description, arguments: args.definitions, returnType: Nothing }
  , invoke: \client state values -> do
      decoded <- orThrow (args.decode (sessionOf client) values)
      encodeJson <$> handler client state decoded
  }

constructor :: forall state a. EncodeJson state => Arguments a -> (Client -> a -> Aff state) -> ModuleFunction state
constructor args handler = stateful "" "" args (\client _ -> handler client)

newtype ObjectDefinition = ObjectDefinition
  { name :: String
  , functions :: Array FunctionDefinition
  , invoke :: Client -> String -> Json -> Object.Object Json -> Aff Json
  }

object_ :: forall state. DecodeJson state => EncodeJson state => String -> state -> Array (ModuleFunction state) -> ObjectDefinition
object_ name initial functions = ObjectDefinition
  { name
  , functions: map functionDefinition allFunctions
  , invoke: \client functionName parent args -> do
      state <- if functionName == "" then pure initial else orThrow (decodeJSON parent)
      case find (\(ModuleFunction f) -> f.definition.name == functionName) allFunctions of
        Nothing -> throwError (error ("Unknown function " <> name <> "." <> functionName))
        Just (ModuleFunction f) -> f.invoke client state args
  }
  where
  allFunctions = if any (\(ModuleFunction f) -> f.definition.name == "") functions then functions
    else functions <> [constructor (pure unit) (\_ _ -> pure initial)]

objectName :: ObjectDefinition -> String
objectName (ObjectDefinition o) = o.name

objectFunctions :: ObjectDefinition -> Array FunctionDefinition
objectFunctions (ObjectDefinition o) = o.functions

functionDefinition :: forall state. ModuleFunction state -> FunctionDefinition
functionDefinition (ModuleFunction f) = f.definition

invokeObject :: Client -> String -> Json -> Object.Object Json -> ObjectDefinition -> Aff Json
invokeObject client name parent args (ObjectDefinition o) = o.invoke client name parent args

orThrow :: forall a. Either String a -> Aff a
orThrow = case _ of
  Left message -> throwError (error message)
  Right a -> pure a
