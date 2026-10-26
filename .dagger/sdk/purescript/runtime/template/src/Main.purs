module Main where

import Prelude
import Dagger (Client)
import Dagger.Container as Container
import Dagger.Modules as Module
import Dagger.Query as Query
import Dagger.Types (Container)
import Effect (Effect)
import Effect.Aff (Aff)

hello :: Client -> String -> Aff String
hello _ name = pure ("Hello, " <> name <> "!")

containerEcho :: Client -> String -> Aff Container
containerEcho client message = pure $ Query.container client
  # Container.from "alpine:3.22"
  # Container.withExec [ "echo", message ]

main :: Effect Unit
main = Module.runModule
  [ Module.object_ "__MODULE_NAME__" {}
      [ Module.function "hello" "Greet someone from PureScript."
          (Module.defaultArgument "name" "world") hello
      , Module.function "container" "Return a container that echoes a message."
          (Module.defaultArgument "message" "Hello from PureScript!") containerEcho
      ]
  ]
