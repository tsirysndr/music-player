import { GitlabCI, Job } from "fluent_gitlab_ci";

export function generateYaml(): GitlabCI {
  const docker = new Job()
    .image("denoland/deno:alpine")
    .services(["docker:${DOCKER_VERSION}-dind"])
    .variables({
      DOCKER_HOST: "tcp://docker:2376",
      DOCKER_TLS_VERIFY: "1",
      DOCKER_TLS_CERTDIR: "/certs",
      DOCKER_CERT_PATH: "/certs/client",
      DOCKER_DRIVER: "overlay2",
      DOCKER_VERSION: "20.10.16",
    });

  const dagger = new Job().extends(".docker").beforeScript(
    `
    apk add docker-cli curl unzip
    deno install -A -r https://cli.fluentci.io -n fluentci
    curl -L https://dl.dagger.io/dagger/install.sh | DAGGER_VERSION=0.8.1 sh
    mv bin/dagger /usr/local/bin
    dagger version
    `
  );

  const tests = new Job()
    .extends(".dagger")
    .script("fluentci run rust_pipeline test build");

  return new GitlabCI()
    .addJob(".docker", docker)
    .addJob(".dagger", dagger)
    .addJob("tests", tests);
}
