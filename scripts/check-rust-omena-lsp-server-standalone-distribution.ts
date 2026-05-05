import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

const packageName = "omena-lsp-server";
const expectedVersion = "0.1.5";
const installCommand = `cargo install ${packageName} --version ${expectedVersion}`;
const splitRepository = "https://github.com/omenien/omena-lsp-server";

try {
  main();
} catch (error) {
  console.error(error);
  process.exit(1);
}

function main() {
  const neovimDoc = readFileSync("docs/clients/neovim.md", "utf8");
  const zedDoc = readFileSync("docs/clients/zed.md", "utf8");

  for (const [label, doc] of [
    ["neovim", neovimDoc],
    ["zed", zedDoc],
  ] as const) {
    assert.match(
      doc,
      /standalone Rust `omena-lsp-server`/u,
      `${label}: must lead with standalone distribution`,
    );
    assert.match(
      doc,
      new RegExp(escapeRegExp(installCommand), "u"),
      `${label}: must document crates.io install`,
    );
    assert.match(
      doc,
      /"omena-lsp-server"|omena-lsp-server/u,
      `${label}: must show the standalone executable`,
    );
    assert.match(
      doc,
      /dist\/bin\/<platform>-<arch>\/omena-lsp-server/u,
      `${label}: must keep repo-local fallback`,
    );
    assert.match(
      doc,
      new RegExp(escapeRegExp(splitRepository), "u"),
      `${label}: must document the standalone split repository`,
    );
  }

  process.stdout.write(
    [
      "validated omena-lsp-server standalone distribution:",
      `package=${packageName}`,
      `version=${expectedVersion}`,
      `docs=neovim,zed`,
    ].join(" "),
  );
  process.stdout.write("\n");
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
