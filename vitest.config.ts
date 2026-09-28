import { configDefaults, defineConfig } from "vitest/config";

const UI = "rust/sens-app/ui/src";
const DOM = [`${UI}/**/*.test.tsx`, `${UI}/bar/**/*.test.{ts,tsx,js}`];

export default defineConfig({
  test: {
    setupFiles: [`${UI}/dev/test-setup.ts`],
    projects: [
      { extends: true, test: { name: "node", include: ["test/**/*.test.ts", `${UI}/**/*.test.{ts,js}`], exclude: [...configDefaults.exclude, ...DOM] } },
      { extends: true, test: { name: "dom", include: DOM, environment: "jsdom" } },
    ],
  },
});
