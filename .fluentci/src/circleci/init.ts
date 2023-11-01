import { generateYaml } from "./config.ts";

generateYaml().save(".circleci/config.yml");
