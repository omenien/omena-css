import path from "node:path";
import react from "@vitejs/plugin-react";
import { omenaCss } from "@omena/vite-plugin";
import { defineConfig } from "vite-plus";

export default defineConfig({
  plugins: [
    omenaCss({
      include: /\.module\.scss$/,
      passes: ["comment-strip"],
      sourceMap: true,
      configFile: false,
    }),
    react(),
  ],
  resolve: {
    alias: {
      $scenarios: path.resolve(__dirname, "src/scenarios"),
    },
  },
  server: { port: 5174 },
});
