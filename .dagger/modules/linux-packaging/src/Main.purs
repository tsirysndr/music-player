module Main where

import Prelude
import Dagger.Modules as Module
import Effect (Effect)
import Release.Deb as Deb
import Release.Rpm as Rpm

main :: Effect Unit
main = Module.runModule
  [ Module.object_ "LinuxPackaging" {}
      [ Module.function "deb" "Build a Debian package from a root containing DEBIAN/control."
          (ado
            root <- Module.argument "root"
            name <- Module.argument "name"
            in { root, name }) Deb.build
      , Module.function "rpm" "Build RPM packages from a spec and SOURCES directory."
          (ado
            sources <- Module.argument "sources"
            spec <- Module.argument "spec"
            target <- Module.defaultArgument "target" "x86_64"
            in { sources, spec, target }) Rpm.build
      ]
  ]
