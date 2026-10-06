import { defineConfig } from "astro/config";

export default defineConfig({
  site: process.env.SITE_URL,
  base: process.env.BASE_PATH,
  build: {
    inlineStylesheets: "auto",
  },
  devToolbar: {
    enabled: false,
  },
});
