import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
const sdk = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const moduleName = process.argv[2];
if (!moduleName) throw new Error("Usage: init-module.mjs <dagger-module-name>");
const objectName = moduleName.split(/[-_\s]+/).map(s => s[0]?.toUpperCase() + s.slice(1)).join("");
if (!/^[A-Z][A-Za-z0-9]*$/.test(objectName)) throw new Error("Invalid Dagger module name");
if (!fs.existsSync("spago.yaml")) {
  fs.writeFileSync("spago.yaml", `package:\n  name: dagger-module\n  dependencies:\n    - aff\n    - dagger-sdk\n    - effect\n    - prelude\nworkspace:\n  packageSet:\n    registry: 77.8.0\n`);
  fs.mkdirSync("src", {recursive:true});
  if (!fs.existsSync("src/Main.purs")) fs.writeFileSync("src/Main.purs", fs.readFileSync(path.join(sdk,"runtime/template/src/Main.purs"),"utf8").replaceAll("__MODULE_NAME__",objectName));
}
if (!fs.existsSync("package.json")) fs.writeFileSync("package.json", JSON.stringify({private:true,type:"module",scripts:{build:"spago build"}},null,2)+"\n");
// A path dependency must not introduce another workspace into the user's project.
const manifest = path.join("dagger-sdk", "spago.yaml");
if (fs.existsSync(manifest)) {
  const source = fs.readFileSync(manifest,"utf8");
  fs.writeFileSync(manifest, source.split(/^workspace:/m)[0].split(/^  test:/m)[0]);
}
