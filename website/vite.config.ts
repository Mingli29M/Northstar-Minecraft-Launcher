import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { copyFileSync, existsSync } from "node:fs";
import { resolve } from "node:path";

/** GitHub Pages SPA fallback: unknown paths serve index.html */
function spaFallback() {
  return {
    name: "spa-fallback",
    closeBundle() {
      const index = resolve(__dirname, "dist/index.html");
      const fallback = resolve(__dirname, "dist/404.html");
      if (existsSync(index)) copyFileSync(index, fallback);
    },
  };
}

export default defineConfig({
  plugins: [react(), tailwindcss(), spaFallback()],
  base: "/Northstar-Minecraft-Launcher/",
});
