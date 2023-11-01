# Deno Pipeline

[![deno module](https://shield.deno.dev/x/deno_pipeline)](https://deno.land/x/deno_pipeline)
![deno compatibility](https://shield.deno.dev/deno/^1.34)
[![](https://img.shields.io/codecov/c/gh/fluent-ci-templates/deno-pipeline)](https://codecov.io/gh/fluent-ci-templates/deno-pipeline)

A ready-to-use CI/CD Pipeline for your Deno projects.

## 🚀 Usage

Run the following command:

```bash
dagger run fluentci deno_pipeline
```

Or, if you want to use it as a template:

```bash
fluentci init -t deno
```

This will create a `.fluentci` folder in your project.

Now you can run the pipeline with:

```bash
dagger run fluentci .
```

## Environment variables (Deno Deploy)

| Variable          | Description               | Default    |
| ----------------- | ------------------------- | ---------- |
| DENO_PROJECT      | Your project name         |            |
| NO_STATIC         | Disable static assets     | `false`    |
| EXCLUDE           | Exclude files from deploy |            |
| DENO_DEPLOY_TOKEN | Your Deno Deploy token    |            |
| DENO_MAIN_SCRIPT  | Your main script          | `main.tsx` |

## Jobs

| Job    | Description                    | Options                |
| ------ | ------------------------------ | ---------------------- |
| fmt    | Format your code               |                        |
| lint   | Lint your code                 |                        |
| test   | Run your tests                 | `{ ignore: string[] }` |
| deploy | Deploy your app to Deno Deploy |                        |

## Programmatic usage

You can also use this pipeline programmatically:

```ts
import { Client, connect } from "https://esm.sh/@dagger.io/dagger@0.8.1";
import { Dagger } from "https://deno.land/x/deno_pipeline/mod.ts";

const { fmt, lint, test } = Dagger;

function pipeline(src = ".") {
  connect(async (client: Client) => {
    await fmt(client, src);
    await lint(client, src);
    await test(client, src);
  });
}

pipeline();
```
