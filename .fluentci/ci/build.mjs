import { buildPlugin } from "@fluentci/inspection";
import { mkdirSync } from "node:fs";
import { spawnSync } from "node:child_process";
function command(program, args) {
  const result = spawnSync(program, args, { stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw Error(`${program} failed with exit status ${result.status}`);
}
// Spago already generates JavaScript, docs.json, and CoreFn for inspection.
command("spago", ["build"]);
mkdirSync("dist", { recursive: true });
await buildPlugin({ sdk: "purescript", runtime: "component" });
