import react from "@vitejs/plugin-react";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { defineConfig, type Plugin } from "vite";
import table from "./src/shared/file-icons.json" with { type: "json" };

export default defineConfig({
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: { rolldownOptions: { input: { main: "index.html", bar: "bar.html" } } },
  worker: { format: "es" },
  plugins: [
    react(),
    fileIcons(),
    {
      name: "sens-mock-tauri",
      apply: "serve",
      transformIndexHtml: () => [
        { tag: "script", attrs: { type: "module", src: "/src/dev/mock-tauri.ts" }, injectTo: "head" },
      ],
    },
  ],
});

function fileIcons(): Plugin {
  const theme = path.dirname(createRequire(import.meta.url).resolve("material-icon-theme/package.json"));
  const manifest = JSON.parse(readFileSync(path.join(theme, "dist", "material-icons.json"), "utf8"));
  const icons = new Set([table.file, ...Object.values(table.names), ...Object.values(table.extensions)]);
  const svg = (icon: string) => readFileSync(path.resolve(theme, "dist", manifest.iconDefinitions[icon].iconPath));

  return {
    name: "sens-file-icons",
    configureServer(server) {
      server.middlewares.use("/file-icons", (request, response, next) => {
        const icon = request.url?.match(/^\/([\w.-]+)\.svg$/)?.[1];
        if (!icon || !icons.has(icon)) return next();
        response.setHeader("Content-Type", "image/svg+xml");
        response.end(svg(icon));
      });
    },
    generateBundle() {
      for (const icon of icons) this.emitFile({ type: "asset", fileName: `file-icons/${icon}.svg`, source: svg(icon) });
      this.emitFile({ type: "asset", fileName: "file-icons/LICENSE", source: readFileSync(path.join(theme, "LICENSE")) });
    },
  };
}
