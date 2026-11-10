module CI.Support where

import Prelude
import Data.Array as A
import Data.Either (either)
import Data.Foldable (foldl)
import Data.Traversable (traverse)
import Data.Maybe (Maybe(..), fromMaybe)
import Data.String as S
import Data.String.CodeUnits as CU
import Effect (Effect)
import Effect.Exception (throw)
import FluentCI.Host.Client (RunOutcome(..), TaskResult)
import FluentCI.Host.Files as F
import FluentCI.Host.Plan as H
import FluentCI.Plugin as P

unwrap :: forall a. P.Result a -> Effect a
unwrap = either (throw <<< P.errorMessage) pure

captureIn :: String -> Array String -> Effect String
captureIn directory command = P.stdout (P.pipeline "command" # P.withWorkdir directory # P.exec command) >>= unwrap

runIn :: String -> Array (Array String) -> Effect Unit
runIn directory commands = void (P.stdout (foldl (flip P.exec) (P.pipeline "commands" # P.withWorkdir directory) commands) >>= unwrap)

run :: Array String -> Effect Unit
run command = runIn "." [ command ]

env :: String -> String -> Effect String
env name fallback = do
  present <- P.hasEnv name >>= unwrap
  if present then P.getEnv name >>= unwrap else pure fallback

requiredEnv :: String -> Effect String
requiredEnv name = do
  value <- env name ""
  if S.trim value == "" then throw ("Missing environment variable: " <> name) else pure value

os :: Effect String
os = S.toLower <$> (P.getOS >>= unwrap)

isMac :: String -> Boolean
isMac value = value == "macos" || value == "darwin"

absolute :: String -> Effect String
absolute directory = S.trim <$> captureIn directory [ "pwd" ]

taskResult :: H.Task -> Effect TaskResult
taskResult task = do
  temporary <- S.trim <$> captureIn "." [ "mktemp", "-d" ]
  result <- P.runPlan (temporary <> "/run") (H.plan [ task ] # H.withInheritedEnv true) >>= unwrap
  unless (result.outcome == Just RunSucceeded) (throw "Native task failed")
  case A.head result.tasks of
    Nothing -> throw "Native task returned no result"
    Just value -> pure value

fileTask :: H.Task -> Effect Unit
fileTask = void <<< taskResult

writeText :: String -> String -> Effect Unit
writeText path content = do
  directory <- S.trim <$> captureIn "." [ "dirname", path ]
  fileTask (F.writeText "write" (basename path) content true directory)

readText :: String -> Effect String
readText path = captureIn "." [ "cat", path ]

files :: String -> Effect (Array String)
files directory = do
  output <- captureIn "." [ "find", directory, "-type", "f" ]
  pure (A.filter (_ /= "") (S.split (S.Pattern "\n") (S.trim output)))

basename :: String -> String
basename path = fromMaybe path (A.last (S.split (S.Pattern "/") path))

checksum :: String -> Effect Unit
checksum path = do
  digest <- S.trim <$> (P.sha256 (P.file path) >>= unwrap)
  writeText (path <> ".sha256") (digest <> "  " <> basename path <> "\n")

validVersion :: String -> Boolean
validVersion value =
  let bare = fromMaybe value (S.stripPrefix (S.Pattern "v") value)
      characters = CU.toCharArray bare
  in case A.head characters of
    Nothing -> false
    Just first -> first >= '0' && first <= '9' && A.all (\c ->
      (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || A.elem c [ '.', '+', '~', '-' ]) characters

-- | Upload all release files, creating the release when absent.
-- | @param directory Release artifact directory.
-- | @param tag Release tag.
-- | @param create Create a release with generated notes if it does not exist.
githubAssets :: { directory :: String, tag :: String, create :: Boolean } -> Effect Unit
githubAssets args = do
  paths <- A.filter isReleaseAsset <$> files args.directory
  when (A.null paths) (throw "No release assets found")
  when args.create do
    -- `gh release create --verify-tag` fails when a release already exists;
    -- inspect the list instead so genuine API/auth errors still fail.
    repository <- requiredEnv "GITHUB_REPOSITORY"
    found <- S.trim <$> captureIn "." [ "gh", "api", "repos/" <> repository <> "/releases", "--paginate", "--jq", ".[] | .tag_name" ]
    unless (A.elem args.tag (S.split (S.Pattern "\n") found)) do
      run [ "gh", "release", "create", args.tag, "--verify-tag", "--generate-notes", "--draft=false", "--prerelease=false" ]
    when (A.elem args.tag (S.split (S.Pattern "\n") found)) do
      run [ "gh", "release", "edit", args.tag, "--draft=false", "--prerelease=false" ]
  run ([ "gh", "release", "upload", args.tag ] <> paths <> [ "--clobber" ])

isReleaseAsset :: String -> Boolean
isReleaseAsset path = A.any (\suffix -> S.endsWith (S.Pattern suffix) path)
  [ ".tar.gz", ".tar.xz", ".tgz", ".zip", ".deb", ".rpm", ".dmg", ".sha256" ]

-- | Configure or push a Cachix cache using host credentials.
-- | @param operation setup or push.
-- | @param cache Cachix cache name.
cachix :: { operation :: String, cache :: String } -> Effect Unit
cachix args = do
  unless (args.operation == "setup" || args.operation == "push") (throw "Expected setup or push")
  let command = [ "nix", "run", "--no-write-lock-file", "--inputs-from", ".", "nixpkgs#cachix", "--" ]
  if args.operation == "setup" then run (command <> [ "use", args.cache ])
  else do
    token <- env "CACHIX_AUTH_TOKEN" ""
    when (S.trim token /= "") (run (command <> [ "push", args.cache, "./result" ]))

-- | Evaluate and build the local Nix flake, then push the result if authenticated.
-- | @param source Source checkout.
-- | @param cache Optional Cachix cache name; empty disables it.
nixBuild :: { source :: String, cache :: String } -> Effect Unit
nixBuild args = do
  when (args.cache /= "") (cachix { operation: "setup", cache: args.cache })
  runIn args.source [ [ "nix", "build", "-L", "--accept-flake-config" ] ]
  when (args.cache /= "") (cachix { operation: "push", cache: args.cache })
