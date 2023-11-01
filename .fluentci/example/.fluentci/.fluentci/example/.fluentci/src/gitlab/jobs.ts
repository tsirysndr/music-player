import { Job } from "https://deno.land/x/fluent_gitlab_ci@v0.3.2/mod.ts";

export const fmt = new Job()
  .image("denoland/deno:alpine")
  .script("deno fmt --check");

export const lint = new Job().image("denoland/deno:alpine").script("deno lint");

export const test = new Job().image("denoland/deno:alpine").script("deno test");
