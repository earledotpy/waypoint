/// <reference types="vitest/config" />
// The line above lets TypeScript accept the `test` section below, which is
// Vitest's. Vitest reads this same file, so tests build the code the same
// way the app does.
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
// @ts-expect-error type error without @types/node package
import process from "node:process";
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [react(), tailwindcss()],

  // `npm test`. jsdom is a web page simulated inside Node, so a component can
  // render and be clicked without opening a window. The setup file runs
  // before each test file.
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test-setup.ts"],
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore the Rust side. `target/` must be ignored too:
      //    it sits at the workspace root, and Vite crashes (EBUSY) when it
      //    tries to watch a file that Cargo has locked while building.
      ignored: ["**/src-tauri/**", "**/crates/**", "**/target/**"],
    },
  },
}));
