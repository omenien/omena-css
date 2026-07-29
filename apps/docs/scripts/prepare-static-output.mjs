import { copyFileSync, existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const applicationRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const publicOutput = path.join(applicationRoot, ".output/public");
const shellPath = path.join(publicOutput, "_shell.html");
const rootIndexPath = path.join(publicOutput, "index.html");
const documentationIndexPath = path.join(publicOutput, "docs/index.html");

if (!existsSync(shellPath)) {
  throw new Error("TanStack Start did not emit the SPA shell");
}

if (!existsSync(documentationIndexPath)) {
  throw new Error("TanStack Start did not prerender the documentation index");
}

if (!existsSync(rootIndexPath)) {
  throw new Error("TanStack Start did not prerender the root documentation route");
}

copyFileSync(shellPath, path.join(publicOutput, "404.html"));
