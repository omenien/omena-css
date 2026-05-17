import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { omenaCss } = require("../packages/vite-plugin/index.cjs");

const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-"));
const stylePath = path.join(tempRoot, "App.module.css");
const warnings = [];

try {
  fs.writeFileSync(stylePath, ".root {\n  color: red;\n}\n/* remove me */\n", "utf8");
  const plugin = omenaCss({
    passes: ["comment-strip", "whitespace-strip"],
    cwd: process.cwd(),
  });
  const input = fs.readFileSync(stylePath, "utf8");
  const result = plugin.transform.call(
    { warn: (message) => warnings.push(message) },
    input,
    stylePath,
  );

  if (!result || typeof result.code !== "string") {
    throw new Error("Expected Vite plugin to return transformed CSS.");
  }
  if (result.code.includes("remove me")) {
    throw new Error(`Expected comment-strip pass to remove comments, got: ${result.code}`);
  }
  if (!result.code.includes(".root")) {
    throw new Error(`Expected transformed CSS to preserve selector, got: ${result.code}`);
  }
  if (warnings.length > 0) {
    throw new Error(`Unexpected Vite plugin warnings: ${warnings.join(" | ")}`);
  }
} finally {
  fs.rmSync(tempRoot, { recursive: true, force: true });
}
