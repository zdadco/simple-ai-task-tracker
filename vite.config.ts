import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Tauri on Windows probes http://127.0.0.1:1420 — bind Vite to IPv4, not [::1] only.
const host = process.env.TAURI_DEV_HOST || "127.0.0.1";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host,
    hmr: process.env.TAURI_DEV_HOST
      ? {
          protocol: "ws",
          host: process.env.TAURI_DEV_HOST,
          port: 1421,
        }
      : {
          protocol: "ws",
          host: "127.0.0.1",
          port: 1421,
        },
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
