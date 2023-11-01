import Client, { connect } from "https://sdk.fluentci.io/v0.1.9/mod.ts";
import {
  build,
  test,
} from "https://pkg.fluentci.io/rust_pipeline@v0.5.2/mod.ts";

function pipeline(src = ".") {
  connect(async (client: Client) => {
    await test(client, src);
    await build(client, src);
  });
}

pipeline();
