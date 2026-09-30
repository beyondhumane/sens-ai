import { defineConfig } from "astro/config";

export default defineConfig({
  build: {
    inlineStylesheets: "auto",
  },
  devToolbar: {
    enabled: false,
  },
});
