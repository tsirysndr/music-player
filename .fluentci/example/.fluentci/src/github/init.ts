import { generateYaml } from "./config.ts";

generateYaml().save(".github/workflows/tests.yml");
