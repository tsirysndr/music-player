import { AzurePipeline } from "fluent_azure_pipelines";

export function generateYaml(): AzurePipeline {
  const azurePipeline = new AzurePipeline();

  const installDeno = `\
  curl -fsSL https://deno.land/x/install/install.sh | sh
  export DENO_INSTALL="$HOME/.deno"
  export PATH="$DENO_INSTALL/bin:$PATH"
`;

  const setupDagger = `\
  curl -L https://dl.dagger.io/dagger/install.sh | DAGGER_VERSION=0.8.1 sh
  sudo mv bin/dagger /usr/local/bin
  dagger version
`;

  azurePipeline
    .trigger(["main"])
    .pool({
      name: "Default",
      vmImage: "ubuntu-latest",
    })
    .step({
      script: installDeno,
      displayName: "Install Deno",
    })
    .step({
      script: "deno install -A -r https://cli.fluentci.io -n fluentci",
      displayName: "Setup Fluent CI CLI",
    })
    .step({
      script: setupDagger,
      displayName: "Setup Dagger",
    })
    .step({
      script: "fluentci run rust_pipeline test build",
      displayName: "Run Dagger Pipelines",
    });
  return azurePipeline;
}
