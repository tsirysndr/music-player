import { assertEquals } from "https://deno.land/std@0.191.0/testing/asserts.ts";
import { generateYaml } from "./config.ts";

Deno.test(function generateAzurePipelinesTest() {
  const azurepipelines = generateYaml();
  const actual = azurepipelines.toString();
  const expected = Deno.readTextFileSync("./fixtures/azure-pipelines.yml");
  assertEquals(actual, expected);
});
