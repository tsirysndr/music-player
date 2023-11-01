import { GitlabCI } from "https://deno.land/x/fluent_gitlab_ci@v0.3.2/mod.ts";
import { fmt, lint, test } from "./jobs.ts";

const pipeline = new GitlabCI()
  .image("denoland/deno:alpine")
  .addJob("fmt", fmt)
  .addJob("lint", lint)
  .addJob("test", test);

export default pipeline;
