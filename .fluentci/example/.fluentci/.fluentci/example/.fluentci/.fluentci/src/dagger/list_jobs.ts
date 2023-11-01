import { brightGreen } from "https://deno.land/std@0.191.0/fmt/colors.ts";
import { runnableJobs, jobDescriptions, Job } from "./jobs.ts";
import { stringifyTree } from "https://esm.sh/stringify-tree@1.1.1";

const tree = {
  name: brightGreen("deno_pipeline"),
  children: (Object.keys(runnableJobs) as Job[]).map((job) => ({
    name: jobDescriptions[job]
      ? `${brightGreen(job)} - ${jobDescriptions[job]}`
      : brightGreen(job),
    children: [],
  })),
};

console.log(
  stringifyTree(
    tree,
    (t) => t.name,
    (t) => t.children
  )
);
