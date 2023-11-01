import Client from "@dagger.io/dagger";
import { existsSync } from "fs";

export enum Job {
  fmt = "fmt",
  lint = "lint",
  test = "test",
}

export const lint = async (client: Client, src = ".") => {
  const context = client.host().directory(src);
  let command = ["deno", "lint"];

  if (existsSync("devbox.json")) {
    command = ["sh", "-c", `devbox run -- ${command.join(" ")}`];
  }

  const ctr = client
    .pipeline(Job.lint)
    .container()
    .from("denoland/deno:alpine")
    .withDirectory("/app", context, {
      exclude: [".git", ".devbox", ".fluentci"],
    })
    .withWorkdir("/app")
    .withExec(command);

  const result = await ctr.stdout();

  console.log(result);
};

export const fmt = async (client: Client, src = ".") => {
  const context = client.host().directory(src);
  let command = ["deno", "fmt"];

  if (existsSync("devbox.json")) {
    command = ["sh", "-c", `devbox run -- ${command.join(" ")}`];
  }

  const ctr = client
    .pipeline(Job.fmt)
    .container()
    .from("denoland/deno:alpine")
    .withDirectory("/app", context, {
      exclude: [".git", ".devbox", ".fluentci"],
    })
    .withWorkdir("/app")
    .withExec(command);

  const result = await ctr.stdout();

  console.log(result);
};

export const test = async (
  client: Client,
  src = ".",
  options: { ignore: string[] } = { ignore: [] }
) => {
  const context = client.host().directory(src);
  let command = ["deno", "test", "-A", "--lock-write"];

  if (options.ignore.length > 0) {
    command = command.concat([`--ignore=${options.ignore.join(",")}`]);
  }

  if (existsSync("devbox.json")) {
    command = ["sh", "-c", `devbox run -- ${command.join(" ")}`];
  }

  const ctr = client
    .pipeline(Job.test)
    .container()
    .from("denoland/deno:alpine")
    .withDirectory("/app", context, {
      exclude: [".git", ".devbox", ".fluentci"],
    })
    .withWorkdir("/app")
    .withMountedCache("/root/.cache/deno", client.cacheVolume("deno-cache"))
    .withExec(command);

  const result = await ctr.stdout();

  console.log(result);
};

export type JobExec = (
  client: Client,
  src?: string
) =>
  | Promise<void>
  | ((
      client: Client,
      src?: string,
      options?: {
        ignore: string[];
      }
    ) => Promise<void>);

export const runnableJobs: Record<Job, JobExec> = {
  [Job.fmt]: fmt,
  [Job.lint]: lint,
  [Job.test]: test,
};

export const jobDescriptions: Record<Job, string> = {
  [Job.fmt]: "Format your code",
  [Job.lint]: "Lint your code",
  [Job.test]: "Run your tests",
};
