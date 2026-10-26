-- | Low-level GraphQL access for queries outside the generated API.
module Dagger.Connection (module Runtime) where

import Dagger.Internal.Runtime (Session, Config, connect, connectWith, defaultConfig, rawQuery) as Runtime
