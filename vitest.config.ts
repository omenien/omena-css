import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    globals: false,
    clearMocks: true,
    coverage: {
      provider: "v8",
      reporter: ["text", "html", "lcov"],
      include: ["shared/src/**", "server/src/**", "client/src/**"],
      exclude: ["**/dist/**", "**/*.d.ts", "**/node_modules/**", "test/**", "**/*.bench.ts"],
      thresholds: {
        lines: 80,
        functions: 80,
        statements: 80,
        branches: 75,
      },
    },
    projects: [
      {
        test: {
          name: "unit",
          include: ["test/unit/**/*.test.ts"],
          setupFiles: ["test/_setup/matchers.ts"],
          testTimeout: 1000,
        },
      },
      {
        test: {
          name: "protocol",
          include: ["test/protocol/**/*.test.ts"],
          setupFiles: ["test/_setup/matchers.ts"],
          testTimeout: 15000,
        },
      },
    ],
  },
});
