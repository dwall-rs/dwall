import path from "node:path";
import { defineConfig } from "vitest/config";

const pathSrc = path.resolve(import.meta.dirname, "src");

// Vitest uses this file *instead of* `vite.config.ts` when it exists, so the path
// aliases are repeated here.
//
// `resolve.conditions` is the part that matters: vitest resolves the `node`
// condition by default, which hands back Solid's server build
// (`solid-js/store/dist/server.js`) — a store that applies updates but is not
// reactive. Tests asserting store state would then be exercising a different
// implementation from the one the app ships.
export default defineConfig({
  resolve: {
    alias: {
      "~/": `${pathSrc}/`,
      "@/": `${pathSrc}/`,
    },
    conditions: ["development", "browser"],
  },
  test: {
    environment: "node",
  },
});
