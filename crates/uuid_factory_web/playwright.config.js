import { defineConfig } from "@playwright/test";

const widths = [375, 768, 1024, 1440];

export default defineConfig({
  testDir: "./tests",
  testMatch: "browser.spec.js",
  fullyParallel: true,
  forbidOnly: true,
  retries: 0,
  reporter: "list",
  use: {
    baseURL: "http://127.0.0.1:4173",
    browserName: "chromium",
    colorScheme: "light",
    reducedMotion: "reduce",
  },
  projects: widths.map((width) => ({
    name: `${width}px`,
    use: { viewport: { width, height: 900 } },
  })),
  webServer: {
    command: "python3 -m http.server 4173 --bind 127.0.0.1 --directory static",
    url: "http://127.0.0.1:4173",
    reuseExistingServer: true,
    timeout: 30_000,
  },
});
