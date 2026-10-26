module Dagger.Module.Value
  ( class DaggerValue, valueType, decodeValue, encodeValue
  , class IsObject, objectTypeName, ValueType(..), decodeJSON
  ) where

import Prelude
import Data.Argonaut.Core (Json, fromArray, isNull, jsonNull)
import Data.Argonaut.Decode (class DecodeJson, decodeJson)
import Data.Argonaut.Encode (encodeJson)
import Data.Either (Either(..))
import Data.Maybe (Maybe(..))
import Data.Traversable (traverse)
import Dagger.Internal.Runtime (Object, Session, argument, execute, fragment, root, select)
import Effect.Aff (Aff)
import Type.Proxy (Proxy(..))

data ValueType
  = StringType
  | IntType
  | NumberType
  | BooleanType
  | VoidType
  | ListType ValueType
  | OptionalType ValueType
  | ObjectType String
  | EnumType String
  | ScalarType String

class DaggerValue a where
  valueType :: Proxy a -> ValueType
  decodeValue :: Session -> Json -> Either String a
  encodeValue :: a -> Aff Json

class IsObject :: Type -> Constraint
class IsObject a where
  objectTypeName :: Proxy a -> String

decodeJSON :: forall a. DecodeJson a => Json -> Either String a
decodeJSON value = case decodeJson value of
  Left err -> Left (show err)
  Right result -> Right result

instance stringValue :: DaggerValue String where
  valueType _ = StringType
  decodeValue _ = decodeJSON
  encodeValue = pure <<< encodeJson

instance intValue :: DaggerValue Int where
  valueType _ = IntType
  decodeValue _ = decodeJSON
  encodeValue = pure <<< encodeJson

instance numberValue :: DaggerValue Number where
  valueType _ = NumberType
  decodeValue _ = decodeJSON
  encodeValue = pure <<< encodeJson

instance booleanValue :: DaggerValue Boolean where
  valueType _ = BooleanType
  decodeValue _ = decodeJSON
  encodeValue = pure <<< encodeJson

instance unitValue :: DaggerValue Unit where
  valueType _ = VoidType
  decodeValue _ value = if isNull value then Right unit else Left "Expected null for Unit"
  encodeValue _ = pure jsonNull

instance arrayValue :: DaggerValue a => DaggerValue (Array a) where
  valueType _ = ListType (valueType (Proxy :: Proxy a))
  decodeValue session value = (decodeJSON value :: Either String (Array Json)) >>= traverse (decodeValue session)
  encodeValue values = fromArray <$> traverse encodeValue values

instance maybeValue :: DaggerValue a => DaggerValue (Maybe a) where
  valueType _ = OptionalType (valueType (Proxy :: Proxy a))
  decodeValue session value = if isNull value then Right Nothing else Just <$> decodeValue session value
  encodeValue = case _ of
    Nothing -> pure jsonNull
    Just value -> encodeValue value

instance objectValue :: IsObject a => DaggerValue (Object a) where
  valueType _ = ObjectType (objectTypeName (Proxy :: Proxy a))
  decodeValue session value = do
    id <- decodeJSON value :: Either String String
    pure (root session # select "node" [argument "id" "ID!" id] # fragment (objectTypeName (Proxy :: Proxy a)))
  encodeValue = execute <<< select "id" []
