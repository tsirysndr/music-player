-- | Author Dagger modules with ordinary typed functions and applicative arguments.
module Dagger.Modules (module Definition, module Runtime, module Value) where

import Dagger.Module.Definition (Arguments, ModuleFunction, ObjectDefinition, argument, defaultArgument, function, method, constructor, stateful, object_) as Definition
import Dagger.Module.Runtime (runModule) as Runtime
import Dagger.Module.Value (class DaggerValue, valueType, decodeValue, encodeValue, ValueType(..), decodeJSON) as Value
