import path from "node:path";
import react from "@vitejs/plugin-react";
import { omenaCss } from "@omena/vite-plugin";
import { defineConfig } from "vite-plus";

const UPSTREAM_VIRTUAL_SOURCE_MARKER = ":root { --omena-upstream-virtual-source: active; }\n";

function upstreamVirtualSource() {
  return {
    name: "examples-upstream-virtual-source",
    enforce: "pre" as const,
    transform(code: string, id: string) {
      if (!/\.module\.scss(?:\?|$)/u.test(id)) return null;
      const sourceLines = code.split(/\r?\n/u);
      return {
        code: `${UPSTREAM_VIRTUAL_SOURCE_MARKER}${code}`,
        map: {
          version: 3,
          file: id,
          sources: [id.split("?", 1)[0]!],
          sourcesContent: [code],
          names: [],
          mappings: ["", ...sourceLines.map((_, index) => (index === 0 ? "AAAA" : "AACA"))].join(
            ";",
          ),
        },
      };
    },
  };
}

function assertVirtualSourceComposition() {
  return {
    name: "examples-virtual-source-composition-check",
    enforce: "pre" as const,
    transform(code: string, id: string) {
      if (!/\.module\.scss(?:\?|$)/u.test(id)) return null;
      if (!code.includes("--omena-upstream-virtual-source: active")) {
        throw new Error(`Omena did not transform the upstream virtual source for ${id}.`);
      }
      return null;
    },
  };
}

export default defineConfig({
  plugins: [
    upstreamVirtualSource(),
    omenaCss({
      include: /\.module\.scss$/,
      passes: ["comment-strip"],
      sourceMap: true,
      configFile: false,
    }),
    assertVirtualSourceComposition(),
    react(),
  ],
  resolve: {
    alias: {
      $scenarios: path.resolve(__dirname, "src/scenarios"),
    },
  },
  server: { port: 5174 },
});
