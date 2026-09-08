import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

/**
 * Test config, kept apart from `vite.config.ts`.
 *
 * The app config pins an output directory and a strict port for the two
 * consumers that depend on them (rust-embed); that does not matter here,
 * and `svgr` is left out because nothing under test imports an SVG as a
 * component. Tailwind stays so a component that renders `@apply`-ed classes
 * does not blow up on the import of `index.css`.
 */
export default defineConfig({
  plugins: [tailwindcss(), react()],
  define: {
    global: "globalThis",
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
    css: true,
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov"],
      // The generated GraphQL hooks and the story fixtures are not code
      // anyone wrote by hand, so they would only dilute the numbers.
      exclude: [
        "src/Hooks/GraphQL.tsx",
        "src/GraphQL/**",
        "src/Stories/**",
        "src/**/*.stories.tsx",
        "src/test/**",
        "src/main.tsx",
        "src/index.tsx",
        "src/reportWebVitals.ts",
      ],
    },
  },
});
