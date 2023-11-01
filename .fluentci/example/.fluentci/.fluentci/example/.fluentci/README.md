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

## Jobs

| Job   | Description      | Options                |
| ----- | ---------------- | ---------------------- |
| fmt   | Format your code |                        |
| lint  | Lint your code   |                        |
| test  | Run your tests   | `{ ignore: string[] }` |

## Programmatic usage

You can also use this pipeline programmatically:

```ts
import Client, { connect } from "@dagger.io/dagger";
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
