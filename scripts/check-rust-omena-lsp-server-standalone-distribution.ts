import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import { readRustPackageMetadata } from "./rust-package-metadata";

const packageMetadata = readRustPackageMetadata("omena-lsp-server");
const installCommand = `cargo install ${packageMetadata.name} --version ${packageMetadata.version}`;

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
      new RegExp(escapeRegExp(packageMetadata.repository), "u"),
      `${label}: must document the crate repository`,
    );
  }

  process.stdout.write(
    [
      "validated omena-lsp-server standalone distribution:",
      `package=${packageMetadata.name}`,
      `version=${packageMetadata.version}`,
      `docs=neovim,zed`,
    ].join(" "),
  );
  process.stdout.write("\n");
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
