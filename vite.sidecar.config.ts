import { defineConfig } from "vite";
import { builtinModules } from "node:module";

/**
 * Builds the Express sidecar into a single CommonJS file at
 * `src-tauri/resources/server.cjs` so Tauri can ship it as a bundled resource.
 *
 * SSR mode + `noExternal: true` bundles all npm dependencies into the output
 * so the Tauri resources directory only needs the one `.cjs` file (no
 * `node_modules/` to ship). Node built-ins are left external.
 */
export default defineConfig({
  build: {
    ssr: true,
    target: "node20",
    outDir: "src-tauri/resources",
    emptyOutDir: false,
    minify: false,
    rollupOptions: {
      input: "src-sidecar/server.ts",
      external: [...builtinModules, ...builtinModules.map((m) => `node:${m}`)],
      output: {
        format: "cjs",
        entryFileNames: "server.cjs",
        inlineDynamicImports: true,
      },
    },
  },
  ssr: {
    noExternal: true,
  },
});
