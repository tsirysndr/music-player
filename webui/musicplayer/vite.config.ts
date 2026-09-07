import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import svgr from "vite-plugin-svgr";

/**
 * Two consumers pin the shape of this build:
 *
 *  - `webui/src/lib.rs` embeds `musicplayer/build/` with rust-embed, and
 *  - `src-tauri/tauri.conf.json` sets `frontendDist: "../build"` and
 *    `devUrl: "http://localhost:3000"`.
 *
 * So the output stays in `build/` rather than Vite's default `dist/`, and the
 * dev server stays on port 3000. Changing either means changing them too.
 */
export default defineConfig({
  plugins: [tailwindcss(), react(), svgr()],
  server: {
    port: 3000,
    // Fail loudly rather than silently moving to 3001, which Tauri's devUrl
    // would then not find.
    strictPort: true,
  },
  build: {
    outDir: "build",
    // CRA emitted no sourcemaps for production either; they would more than
    // double what rust-embed bakes into the binary.
    sourcemap: false,
  },
  // `global` is referenced by a few transitive deps written for webpack.
  define: {
    global: "globalThis",
  },
});
