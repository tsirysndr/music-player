module CI.Gemfury (upload, uploadFiles) where

import Prelude
import Data.Array as Array
import Data.Either (Either(..))
import Data.Foldable (all, traverse_)
import Data.String.CodeUnits as CU
import Data.String.Common (replaceAll, trim)
import Data.String.Pattern (Pattern(..), Replacement(..))
import Effect (Effect)
import Effect.Exception (throw)
import FluentCI.Plugin as P

unwrap :: forall a. P.Result a -> Effect a
unwrap = case _ of
  Left error -> throw (P.errorMessage error)
  Right value -> pure value

requiredEnv :: String -> Effect String
requiredEnv name = do
  present <- P.hasEnv name >>= unwrap
  value <- if present then P.getEnv name >>= unwrap else pure ""
  if trim value == "" then throw ("Missing required environment variable: " <> name)
  else pure value

withToken :: Effect Unit -> Effect Unit
withToken action = do
  present <- P.hasEnv "FURY_TOKEN" >>= unwrap
  token <- if present then P.getEnv "FURY_TOKEN" >>= unwrap else pure ""
  accountPresent <- P.hasEnv "FURY_ACCOUNT" >>= unwrap
  account <- if accountPresent then P.getEnv "FURY_ACCOUNT" >>= unwrap else pure ""
  if trim token == "" || trim account == "" then
    P.output "Skipping Gemfury upload: FURY_TOKEN or FURY_ACCOUNT is not set." >>= unwrap
  else action

run :: Array String -> Effect Unit
run args = void (P.pipeline "gemfury-upload" # P.exec args # P.stdout >>= unwrap)

-- Curl's multipart grammar has its own quoting rules in addition to argv.
formPath :: String -> String
formPath path = "package=@\"" <> escaped <> "\""
  where
  -- Curl treats @- as stdin, even when quoted. Refer to that filename explicitly.
  escaped = (if path == "-" then "./-" else path)
    # replaceAll (Pattern "\\") (Replacement "\\\\")
    # replaceAll (Pattern "\"") (Replacement "\\\"")

validatePath :: String -> Effect Unit
validatePath path = do
  unless (trim path /= "" && all (\c -> c /= '\x0000' && c /= '\n' && c /= '\r') (CU.toCharArray path)) $
    throw "Package paths must be nonempty and cannot contain NUL or line breaks"

-- Credentials remain environment references in argv, without shell expansion.
publish :: Array String -> Effect Unit
publish paths = do
  when (Array.null paths) $ throw "At least one package path is required"
  traverse_ validatePath paths
  account <- requiredEnv "FURY_ACCOUNT"
  let alphaNum c = (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9')
  unless (all (\c -> alphaNum c || c == '-' || c == '_') (CU.toCharArray account)) $
    throw "Invalid Gemfury account name"
  -- Preflight the entire batch before the first network request.
  traverse_ (\path -> do
    run [ "test", "-f", path ]
    run [ "test", "-r", path ]
    run [ "test", "-s", path ]) paths
  traverse_ (\path -> run
    [ "curl", "--fail-with-body", "--silent", "--show-error"
    , "--variable", "%FURY_TOKEN"
    , "--expand-user", "{{FURY_TOKEN}}:"
    , "--form", formPath path, "https://push.fury.io/" <> account <> "/" ]) paths

uploadFiles :: Array String -> Effect Unit
uploadFiles paths = withToken (publish paths)

-- CLI input is one literal path, with FURY_PACKAGE as an environment fallback.
upload :: Effect Unit
upload = withToken do
  input <- P.input >>= unwrap
  path <- if trim input == "" then requiredEnv "FURY_PACKAGE" else pure input
  publish [ path ]
