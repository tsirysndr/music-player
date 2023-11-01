import pipeline from "./pipeline.ts";
import { assertEquals } from "https://deno.land/std@0.191.0/testing/asserts.ts";

Deno.test(function pipelineTest() {
  const expected = Deno.readTextFileSync("./fixtures/.gitlab-ci.yml");
  const actual = pipeline.toString();
  assertEquals(actual, expected);
});
