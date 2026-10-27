import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Widget Tauri: bundle estático; el core Rust no expone puertos (constitución IV).
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // El target de Rust (PDB en escritura por cargo) mata el watcher en
      // Windows con EBUSY — tauri dev ya reconstruye el core por su cuenta.
      ignored: ["**/src-tauri/**", "**/dist/**", "**/node_modules/**"],
    },
  },
  build: {
    target: "es2022",
    minify: true,
    chunkSizeWarningLimit: 700,
  },
});
