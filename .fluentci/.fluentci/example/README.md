# Deno Pipeline Example

This is an example using the [Deno Pipeline](https://github.com/fluent-ci-templates/deno-pipeline).

## 🚀 Usage

You need to set the following environment variables:

- `DENO_DEPLOY_TOKEN`: Your Deno Deploy token.
- `DENO_PROJECT`: Your project name.

Then, run the following command:

```bash
dagger run fluentci . fmt lint deploy
```