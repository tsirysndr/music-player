module Dagger
  ( Client, connect, connectWith, withConnection, module Connection
  ) where

import Prelude
import Dagger.Internal.Runtime (Config, defaultConfig) as Connection
import Dagger.Internal.Runtime as Runtime
import Dagger.Types (Query)
import Effect (Effect)
import Effect.Aff (Aff)
import Effect.Class (liftEffect)

type Client = Query

-- | Attach to the session owned by `dagger run`.
connect :: Effect Client
connect = Runtime.root <$> Runtime.connect

connectWith :: Connection.Config -> Effect Client
connectWith config = Runtime.root <$> Runtime.connectWith config

-- | The CLI owns the engine session; individual requests manage cancellation.
withConnection :: forall a. (Client -> Aff a) -> Aff a
withConnection use = liftEffect connect >>= use
