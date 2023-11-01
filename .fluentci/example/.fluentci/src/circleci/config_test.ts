import { assertEquals } from "https://deno.land/std@0.191.0/testing/asserts.ts";
import { generateYaml } from "./config.ts";

Deno.test(function generateCircleCITest() {
  const circleci = generateYaml();
  const actual = circleci.toString();
  const expected = Deno.readTextFileSync("./fixtures/config.yml");
  assertEquals(actual, expected);
});
