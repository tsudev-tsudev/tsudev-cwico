import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Tauri serves the built assets from disk with a `tauri://` origin, so every
// path has to be relative rather than rooted at `/`.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  base: "./",
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      // The Rust side has its own rebuild loop; watching it here just burns
      // file descriptors.
      ignored: ["**/src-tauri/**", "**/target/**"],
    },
  },
  build: {
    outDir: "dist",
    // Windows 10's WebView2 tracks Edge, so a modern baseline is safe.
    target: "chrome110",
    sourcemap: false,
    emptyOutDir: true,
  },
});
