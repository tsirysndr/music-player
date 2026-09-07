import { setupServer } from "msw/node";
import { handlers } from "./handlers";

/**
 * The MSW server, started once in `setup.ts`.
 *
 * Exported so a test can narrow a single operation with `server.use(...)`
 * without restating everything else the page happens to fetch.
 */
export const server = setupServer(...handlers);
