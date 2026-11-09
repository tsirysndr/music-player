module CI.DuckDB (fetch, nativeTarget) where

import Prelude
import CI.Support as S
import Data.Array as A
import Data.String as String
import Effect (Effect)
import Effect.Exception (throw)
import FluentCI.Plugin as P
import FluentCI.Host.Files as F

nativeTarget :: Effect String
nativeTarget = do
  host <- S.os
  arch <- P.getArch >>= S.unwrap
  let arm = A.elem arch [ "aarch64", "arm64" ]
      cpu = if arm then "aarch64" else "x86_64"
  pure (cpu <> if S.isMac host then "-apple-darwin" else "-unknown-linux-gnu")

fetch :: String -> String -> Effect Unit
fetch source target = do
  platform <- case target of
    "aarch64-apple-darwin" -> pure "osx-arm64"
    "x86_64-apple-darwin" -> pure "osx-amd64"
    "aarch64-unknown-linux-gnu" -> pure "linux-arm64"
    "x86_64-unknown-linux-gnu" -> pure "linux-amd64"
    _ -> throw "No static DuckDB archive for this target"
  let version = "1.5.5"
      directory = source <> "/vendor/duckdb/lib"
      stamp = version <> "-" <> platform
      url = "https://github.com/duckdb/duckdb/releases/download/v" <> version <> "/static-libs-" <> platform <> ".zip"
  S.run [ "mkdir", "-p", directory ]
  existing <- S.files directory
  previous <- if A.elem (directory <> "/.stamp") existing then S.readText (directory <> "/.stamp") else pure ""
  unless (previous == stamp && A.elem (directory <> "/libduckdb_static.a") existing) do
    temporary <- String.trim <$> S.captureIn "." [ "mktemp", "-d" ]
    S.run [ "curl", "--fail", "--location", "--output", temporary <> "/duckdb.zip", url ]
    S.fileTask (F.remove "clear-duckdb" "lib" true true (source <> "/vendor/duckdb"))
    S.run [ "mkdir", "-p", directory ]
    S.run [ "unzip", "-q", "-j", temporary <> "/duckdb.zip", "*.a", "duckdb.h", "-d", directory ]
    S.run [ "test", "-f", directory <> "/libduckdb_static.a" ]
    -- Written last: the build script trusts this to indicate a complete set.
    S.writeText (directory <> "/.stamp") stamp
