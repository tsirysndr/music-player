// Setup environment updates normally last only one FluentCI invocation.
import { appendFileSync } from "node:fs";
import { randomUUID } from "node:crypto";

export function setupEnvironment(output) {
  const lines = output.trimEnd().split("\n");
  for (let start = lines.length - 1; start >= 0; start--) {
    let result;
    try { result = JSON.parse(lines.slice(start).join("\n")); } catch { continue; }
    if (!Array.isArray(result.environment)) continue;
    for (const { name, value } of result.environment) {
      if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(name) || typeof value !== "string" || value.includes("\0")) {
        throw Error("Invalid setup environment entry");
      }
    }
    return result.environment;
  }
  throw Error("Setup did not return a native environment result");
}

export function persistEnvironment(output, destination) {
  const entries = setupEnvironment(output);
  const delimiter = `fluentci_${randomUUID()}`;
  appendFileSync(destination, entries.map(({ name, value }) => `${name}<<${delimiter}\n${value}\n${delimiter}\n`).join(""));
}

if (process.argv[1] === new URL(import.meta.url).pathname) {
  let output = "";
  for await (const chunk of process.stdin) {
    process.stdout.write(chunk);
    output += chunk;
  }
  if (!process.env.GITHUB_ENV) throw Error("GITHUB_ENV is required");
  persistEnvironment(output, process.env.GITHUB_ENV);
}
